local ROWS = 34
local COLS = 9
local NUM_STARS = math.floor(ROWS * COLS * 0.10)
local MAX_SUPER = 2

math.randomseed(os.time())

local function shuffle(t)
    for i = #t, 2, -1 do
        local j = math.random(1, i)
        t[i], t[j] = t[j], t[i]
    end
end

local function make_stars()
    local positions = {}
    for r = 0, ROWS - 1 do
        for c = 0, COLS - 1 do
            positions[#positions + 1] = {r, c}
        end
    end
    shuffle(positions)
    local out = {}
    for i = 1, NUM_STARS do
        out[i] = {
            row        = positions[i][1],
            col        = positions[i][2],
            phase      = math.random() * 2 * math.pi,
            speed      = 0.06 + math.random() * 0.12,  -- radians/tick → period ~2-5s
            base_b     = math.random(10, 40),
            peak_b     = math.random(100, 255),
            super_ticks = 0,
            super_max   = 0,
        }
    end
    return out
end

local stars = make_stars()

local function px(frame, r, c, b)
    if r >= 0 and r < ROWS and c >= 0 and c < COLS then
        frame:set(r, c, b)
    end
end

function desired_interval_ms()
    return 80
end

function tick(dt_ms, frame)
    -- count active super-twinkles and maybe start a new one
    local active = 0
    for _, s in ipairs(stars) do
        if s.super_ticks > 0 then active = active + 1 end
    end

    if active < MAX_SUPER and math.random() < 0.06 then
        local pool = {}
        for i, s in ipairs(stars) do
            if s.super_ticks == 0 then pool[#pool + 1] = i end
        end
        if #pool > 0 then
            local s = stars[pool[math.random(#pool)]]
            s.super_ticks = 6
            s.super_max   = 6
        end
    end

    for _, s in ipairs(stars) do
        s.phase = s.phase + s.speed
        local t = (math.sin(s.phase) + 1) * 0.5              -- 0..1
        local b = math.floor(s.base_b + (s.peak_b - s.base_b) * t)

        if s.super_ticks > 0 then
            local frac   = s.super_ticks / s.super_max        -- 1.0 → fades to 0
            local center = math.min(255, math.floor(b + (255 - b) * frac))
            local arm1   = math.floor(center * 0.70 * frac)   -- adjacent cardinal
            local arm2   = math.floor(center * 0.40 * frac)   -- outer cardinal
            local diag   = math.floor(center * 0.25 * frac)   -- outer diagonal (skip 1)
            local r, c = s.row, s.col
            -- center
            px(frame, r,     c,     center)
            -- cardinal arms, 2 deep
            px(frame, r - 1, c,     arm1)
            px(frame, r - 2, c,     arm2)
            px(frame, r + 1, c,     arm1)
            px(frame, r + 2, c,     arm2)
            px(frame, r,     c - 1, arm1)
            px(frame, r,     c - 2, arm2)
            px(frame, r,     c + 1, arm1)
            px(frame, r,     c + 2, arm2)
            -- diagonals: skip distance-1, light distance-2
            px(frame, r - 2, c - 2, diag)
            px(frame, r - 2, c + 2, diag)
            px(frame, r + 2, c - 2, diag)
            px(frame, r + 2, c + 2, diag)
            s.super_ticks = s.super_ticks - 1
        else
            px(frame, s.row, s.col, b)
        end
    end
    return true
end
