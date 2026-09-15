-- Usage: mame genesis -cart <path> -autoboot_script plugins/record.lua
package.path = package.path .. ";./plugins/?.lua"

local controller = require('controller')
local reader = require('reader')
local c = require('common')

local cpu = manager.machine.devices[":maincpu"]
local mem = cpu.spaces["program"]
local path = os.getenv('RECORDING') or 'output.mlen'
local f = io.open(path, 'wb')

if f == nil then
  emu.print_error('Could not open file: ' .. path)
  manager.machine:exit()
end

r = reader.new(controller.new(c.IO_DATA, c.IO_CTRL))
frame = 0

local function handle_write(address, data, mask)
  r:on_write(address, data, mask)
end

local function handle_read(address, data, mask)
  r:on_read(address, data, mask)
  if r.changed then
    local keys = r:keys()
    io.write(string.format('frame=%d down=%s up=%s left=%s right=%s start=%s a=%s b=%s c=%s\n',
      frame, keys.down, keys.up, keys.left, keys.right, keys.start, keys.a, keys.b, keys.c))
    assert(f ~= nil)

    -- Recording only contains key state changes using the following binary format:
    --
    -- <frame_number:u32><player_1_keys:u8><player_2_keys:u8><player_3_keys:u8><player_4_keys:u8>
    --
    -- This years' game is single player only, so only player 1 state is stored.
    -- The frame number is stored in little endian.
    f:write(c.pack_dword(frame) .. c.pack_dword(c.to_storage(r:keys())))
  end
  if r.ready then
    frame = frame + 1
  end
end

_G.record_tap1 = mem:install_read_tap(c.IO_BASE_START, c.IO_BASE_END, "record read", handle_read)
_G.record_tap2 = mem:install_write_tap(c.IO_BASE_START, c.IO_BASE_END, "record write", handle_write)

local on_exit = emu.register_stop
if emu['add_machine_stop_notifier'] ~= nil then
  on_exit = emu.add_machine_stop_notifier
end
_G.record_exit_hook = on_exit(function()
  if f ~= nil then
    f:write(c.pack_dword(frame) .. c.pack_dword(0))
    f:close()
  end
end)
