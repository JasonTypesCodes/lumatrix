local ROWS = 34
local COLS = 9
local NUM_DROPS = 6
local TRAIL = {255, 160, 80, 20}

math.randomseed(os.time())

local drops = {}
for i = 1, NUM_DROPS do
    drops[i] = { col = math.random(0, COLS - 1), row = -(i - 1) * 7 }
end

function desired_interval_ms()
    return 100
end

function tick(dt_ms, frame)
    for i = 1, NUM_DROPS do
        local d = drops[i]
        d.row = d.row + 1
        if d.row > ROWS + #TRAIL then
            d.col = math.random(0, COLS - 1)
            d.row = 0
        end
        for j, b in ipairs(TRAIL) do
            local r = d.row - (j - 1)
            if r >= 0 and r < ROWS then
                frame:set(r, d.col, b)
            end
        end
    end
    return true
end
