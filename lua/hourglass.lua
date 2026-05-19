local ROWS = 34
local COLS = 9
local DURATION_MS = (tonumber(args and args[1]) or 30) * 1000
local OUTLINE_B = 255
local FILL_B = 85
local CHECK_B = 255
local CHECKMARK_DURATION_MS = 2000

local elapsed_ms = 0
local phase = "filling"  -- "filling", "checkmark", "done"
local checkmark_ms = 0

local function half_row(r)
    if r <= 16 then return r else return 33 - r end
end

local function lc(r)
    return math.floor(half_row(r) * 3 / 16 + 0.5)
end

local function rc(r)
    return COLS - 1 - lc(r)
end

-- Checkmark: tip at (20,2), left arm up-left to (18,0), right arm up-right to (14,8)
local CHECKMARK = {
    {18, 0}, {19, 1}, {20, 2},
    {19, 3}, {18, 4}, {17, 5}, {16, 6}, {15, 7}, {14, 8},
}

function is_done()
    return phase == "done"
end

function desired_interval_ms()
    return 100
end

function tick(dt_ms, frame)
    if phase == "filling" then
        elapsed_ms = math.min(elapsed_ms + dt_ms, DURATION_MS)

        if elapsed_ms >= DURATION_MS then
            -- Transition on the same tick so the checkmark appears at exactly 30s.
            phase = "checkmark"
        else
            local progress = elapsed_ms / DURATION_MS

            local top_filled    = math.floor(17 * (1 - progress) + 0.5)
            local bottom_filled = 17 - top_filled
            local top_start     = 17 - top_filled
            local bottom_start  = 34 - bottom_filled

            -- Top half: gravity pulls sand DOWN, so it sits at the bottom of the
            -- top chamber (rows nearest the waist). Empty space grows from the top.
            for r = top_start, 16 do
                if rc(r) - lc(r) > 2 then  -- skip single-pixel waist pinch
                    for c = lc(r) + 1, rc(r) - 1 do
                        frame:set(r, c, FILL_B)
                    end
                end
            end

            -- Bottom half: sand accumulates at the bottom (row 33) and grows upward.
            for r = bottom_start, ROWS - 1 do
                if rc(r) - lc(r) > 2 then  -- skip single-pixel waist pinch
                    for c = lc(r) + 1, rc(r) - 1 do
                        frame:set(r, c, FILL_B)
                    end
                end
            end

            -- Outline always at full brightness
            for r = 0, ROWS - 1 do
                frame:set(r, lc(r), OUTLINE_B)
                frame:set(r, rc(r), OUTLINE_B)
            end

            return true
        end
    end

    if phase == "checkmark" then
        checkmark_ms = checkmark_ms + dt_ms
        for _, px in ipairs(CHECKMARK) do
            frame:set(px[1], px[2], CHECK_B)
        end
        if checkmark_ms >= CHECKMARK_DURATION_MS then
            phase = "done"
        end

    elseif phase == "done" then
        return false
    end

    return true
end
