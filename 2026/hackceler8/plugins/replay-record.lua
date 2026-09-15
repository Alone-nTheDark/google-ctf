-- Usage: mame genesis -cart <path> -autoboot_script plugins/replay-record.lua
package.path = package.path .. ";./plugins/?.lua"

require('replay')
require('record')
