local ROWS = 34
local COLS = 9

local DIRS = {
    {-1, 0}, {1, 0}, {0, -1}, {0, 1},
    {-1, -1}, {-1, 1}, {1, -1}, {1, 1},
}

local MAX_STEPS = 7
local STEP_BRIGHT = {255, 245, 220, 185, 140, 85, 35}
local TRAIL_FRAC = {1.0, 0.45, 0.18, 0.06}

math.randomseed(os.time())

local function rand_col() return math.random(1, 7) end
local function rand_row() return math.random(10, 20) end

local function new_firework(delay)
    return {
        phase = "waiting",
        delay = delay,
        burst_row = rand_row(),
        burst_col = rand_col(),
        rocket_row = ROWS - 1,
        burst_frames = 0,
        expand_step = 0,
    }
end

local fireworks = {}
local count = math.random(3, 5)
for i = 1, count do
    fireworks[i] = new_firework(math.random(0, 12))
end

local function px(frame, r, c, b)
    if r >= 0 and r < ROWS and c >= 0 and c < COLS then
        frame:set(r, c, b)
    end
end

local function tick_firework(fw, frame)
    if fw.phase == "waiting" then
        fw.delay = fw.delay - 1
        if fw.delay <= 0 then
            fw.phase = "launching"
        end

    elseif fw.phase == "launching" then
        px(frame, fw.rocket_row,     fw.burst_col, 220)
        px(frame, fw.rocket_row + 1, fw.burst_col, 90)
        px(frame, fw.rocket_row + 2, fw.burst_col, 30)
        fw.rocket_row = fw.rocket_row - 1
        if fw.rocket_row < fw.burst_row then
            fw.phase = "bursting"
            fw.burst_frames = 2
        end

    elseif fw.phase == "bursting" then
        local br, bc = fw.burst_row, fw.burst_col
        px(frame, br,     bc,     255)
        px(frame, br - 1, bc,     210)
        px(frame, br + 1, bc,     210)
        px(frame, br,     bc - 1, 210)
        px(frame, br,     bc + 1, 210)
        px(frame, br - 1, bc - 1, 120)
        px(frame, br - 1, bc + 1, 120)
        px(frame, br + 1, bc - 1, 120)
        px(frame, br + 1, bc + 1, 120)
        fw.burst_frames = fw.burst_frames - 1
        if fw.burst_frames == 0 then
            fw.phase = "expanding"
            fw.expand_step = 1
        end

    elseif fw.phase == "expanding" then
        local lead_b = STEP_BRIGHT[fw.expand_step] or 0
        local br, bc = fw.burst_row, fw.burst_col
        for _, dir in ipairs(DIRS) do
            for trail_i, frac in ipairs(TRAIL_FRAC) do
                local t = fw.expand_step - (trail_i - 1)
                if t >= 1 then
                    local b = math.floor(lead_b * frac)
                    if b > 0 then
                        px(frame, br + dir[1] * t, bc + dir[2] * t, b)
                    end
                end
            end
        end
        fw.expand_step = fw.expand_step + 1
        if fw.expand_step > MAX_STEPS then
            fw.phase = "waiting"
            fw.delay = math.random(0, 12)
            fw.burst_row = rand_row()
            fw.burst_col = rand_col()
            fw.rocket_row = ROWS - 1
            fw.burst_frames = 0
            fw.expand_step = 0
        end
    end
end

function desired_interval_ms()
    return 80
end

function tick(dt_ms, frame)
    for _, fw in ipairs(fireworks) do
        tick_firework(fw, frame)
    end
    return true
end
