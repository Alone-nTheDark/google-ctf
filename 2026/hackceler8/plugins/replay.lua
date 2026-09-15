-- Usage: mame genesis -cart <path> -autoboot_script plugins/replay.lua
package.path = package.path .. ";./plugins/?.lua"

local controller = require('controller')
local writer = require('writer')
local c = require('common')

local cpu = manager.machine.devices[":maincpu"]
local mem = cpu.spaces["program"]
local path = os.getenv('REPLAY') or 'output.mlen'
local f = io.open(path, 'rb')

if f == nil then
  emu.print_error('Could not open file: ' .. path)
  manager.machine:exit()
end
assert(f ~= nil)

local function read_next_frame(input)
  local raw_frame_number = input:read(4)
  if raw_frame_number == nil then
    return nil, nil
  end

  local keys = c.from_storage(c.unpack_dword(input:read(4)))
  return c.unpack_dword(raw_frame_number), keys
end

local w = writer.new(controller.new(c.IO_DATA, c.IO_CTRL))
local frame = 0
local next_frame, next_keys = read_next_frame(f)
assert(next_keys ~= nil)

local function handle_write(address, data, mask)
  w:on_write(address, data, mask)
end

local function handle_read(address, data, mask)
  if next_frame == nil then
    return nil
  end

  if frame == next_frame then
    io.write(string.format('frame=%d down=%s up=%s left=%s right=%s start=%s a=%s b=%s c=%s\n',
      next_frame, next_keys.down, next_keys.up, next_keys.left, next_keys.right, next_keys.start, next_keys.a,
      next_keys.b, next_keys.c))
    w:set_keys(next_keys)
    next_frame, next_keys = read_next_frame(f)
    if next_frame == nil then
      emu.pause()
    end
  end
  local value = w:on_read(address, data, mask)
  if w.ready then
    frame = frame + 1
  end
  return value
end

_G.replay_tap1 = mem:install_read_tap(c.IO_BASE_START, c.IO_BASE_END, "replay read", handle_read)
_G.replay_tap2 = mem:install_write_tap(c.IO_BASE_START, c.IO_BASE_END, "replay write", handle_write)
_G.replay_exit_hook = emu.register_stop(function()
  if f ~= nil then
    f:close()
  end
end)
