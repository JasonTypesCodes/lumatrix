-- play.lua — play a hand-authored animation file on the LED matrix
-- Usage: lumatrix load play /path/to/animation.lmx
--
-- File format:
--   # comment lines (ignored everywhere)
--   fps 10        (frames per second; default 10)
--   ms 100        (alternative: milliseconds per frame)
--   loop no       (disable looping; default loops forever)
--   ---           (starts each frame; repeat for each frame)
--   #########     (pixel row: 1 char per LED column, up to 9)
--   ...
--
-- Brightness characters:
--   ' '  = 0   (off)
--   '.'  = 50  (faint)
--   '+'  = 120 (medium)
--   '#'  = 200 (bright)
--   '@'  = 255 (full)

local BRIGHTNESS = {
    [" "] = 0,
    ["."] = 50,
    ["+"] = 120,
    ["#"] = 200,
    ["@"] = 255,
}

local path = args and args[1]
if not path then
    error("play.lua: no file specified — usage: lumatrix load play /path/to/animation.lmx")
end

local file, err = io.open(path, "r")
if not file then
    error("play.lua: cannot open '" .. path .. "': " .. tostring(err))
end

local fps_val = 10
local do_loop = true
local frames = {}
local current = nil  -- nil until first --- is seen

for line in file:lines() do
    local stripped = line:match("^(.-)%s*$")  -- trim trailing whitespace

    -- strip inline comments (but only outside frame rows, i.e. before first ---)
    if current == nil then
        stripped = stripped:match("^([^#]*)") or ""
        stripped = stripped:match("^(.-)%s*$")
    end

    if stripped == "---" then
        if current ~= nil then
            table.insert(frames, current)
        end
        current = {}
    elseif current ~= nil then
        -- inside a frame: '#' is a pixel char, not a comment
        table.insert(current, stripped)
    elseif stripped:match("^fps%s+") then
        fps_val = tonumber(stripped:match("^fps%s+(%d+)")) or fps_val
    elseif stripped:match("^ms%s+") then
        local ms = tonumber(stripped:match("^ms%s+(%d+)"))
        if ms and ms > 0 then fps_val = 1000 / ms end
    elseif stripped:match("^loop%s+no") then
        do_loop = false
    end
end
file:close()

if current ~= nil then
    table.insert(frames, current)
end

if #frames == 0 then
    error("play.lua: no frames found in '" .. path .. "'")
end

local frame_idx = 1
local done = false

function desired_interval_ms()
    return math.max(30, math.floor(1000 / fps_val))
end

function tick(dt_ms, frame)
    local rows = frames[frame_idx]
    for r = 0, 33 do
        local row_str = rows[r + 1] or ""
        for c = 0, 8 do
            local ch = row_str:sub(c + 1, c + 1)
            frame:set(r, c, BRIGHTNESS[ch] or 0)
        end
    end

    frame_idx = frame_idx + 1
    if frame_idx > #frames then
        if do_loop then
            frame_idx = 1
        else
            done = true
        end
    end

    return true
end

function is_done()
    return done
end
