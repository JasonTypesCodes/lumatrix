-- clock.lua — current time displayed sideways as HH:MM
-- Each digit is a 5-wide × 7-tall pixel bitmap rendered rotated 90°.
-- The 7-row character height spans matrix cols 1–7 (centred in 9).
-- The 5-col character width spans 5 matrix rows per digit.
--
-- Layout (34 rows total):
--   rows  0– 4  padding
--   rows  5– 9  H1 (tens of hours)
--   row  10      gap
--   rows 11–15  H2 (units of hours)
--   row  16      gap
--   row  17      colon (two pixels, blinking)
--   row  18      gap
--   rows 19–23  M1 (tens of minutes)
--   row  24      gap
--   rows 25–29  M2 (units of minutes)
--   rows 30–33  padding

-- 5×7 bitmaps: DIGITS[d][char_row][char_col], both 1-indexed.
-- char_row 1 = top of digit, char_row 7 = bottom.
-- char_col 1 = left of digit, char_col 5 = right.
-- When rendered sideways: char_row → matrix col offset, char_col → matrix row offset.
local DIGITS = {
    [0] = {
        {0,1,1,1,0},
        {1,0,0,0,1},
        {1,0,0,0,1},
        {1,0,0,0,1},
        {1,0,0,0,1},
        {1,0,0,0,1},
        {0,1,1,1,0},
    },
    [1] = {
        {0,0,1,0,0},
        {0,1,1,0,0},
        {0,0,1,0,0},
        {0,0,1,0,0},
        {0,0,1,0,0},
        {0,0,1,0,0},
        {0,1,1,1,0},
    },
    [2] = {
        {0,1,1,1,0},
        {1,0,0,0,1},
        {0,0,0,0,1},
        {0,0,1,1,0},
        {0,1,0,0,0},
        {1,0,0,0,0},
        {1,1,1,1,1},
    },
    [3] = {
        {0,1,1,1,0},
        {1,0,0,0,1},
        {0,0,0,0,1},
        {0,0,1,1,0},
        {0,0,0,0,1},
        {1,0,0,0,1},
        {0,1,1,1,0},
    },
    [4] = {
        {0,0,0,1,0},
        {0,0,1,1,0},
        {0,1,0,1,0},
        {1,0,0,1,0},
        {1,1,1,1,1},
        {0,0,0,1,0},
        {0,0,0,1,0},
    },
    [5] = {
        {1,1,1,1,1},
        {1,0,0,0,0},
        {1,1,1,1,0},
        {0,0,0,0,1},
        {0,0,0,0,1},
        {1,0,0,0,1},
        {0,1,1,1,0},
    },
    [6] = {
        {0,1,1,1,0},
        {1,0,0,0,0},
        {1,0,0,0,0},
        {1,1,1,1,0},
        {1,0,0,0,1},
        {1,0,0,0,1},
        {0,1,1,1,0},
    },
    [7] = {
        {1,1,1,1,1},
        {0,0,0,0,1},
        {0,0,0,1,0},
        {0,0,1,0,0},
        {0,1,0,0,0},
        {0,1,0,0,0},
        {0,1,0,0,0},
    },
    [8] = {
        {0,1,1,1,0},
        {1,0,0,0,1},
        {1,0,0,0,1},
        {0,1,1,1,0},
        {1,0,0,0,1},
        {1,0,0,0,1},
        {0,1,1,1,0},
    },
    [9] = {
        {0,1,1,1,0},
        {1,0,0,0,1},
        {1,0,0,0,1},
        {0,1,1,1,1},
        {0,0,0,0,1},
        {0,0,0,0,1},
        {0,1,1,1,0},
    },
}

local BRIGHTNESS = 200

local function draw_digit(frame, start_row, d)
    local bitmap = DIGITS[d]
    for ci = 1, 7 do
        for ri = 1, 5 do
            if bitmap[ci][ri] == 1 then
                frame:set(start_row + ri - 1, 8 - ci, BRIGHTNESS)
            end
        end
    end
end

function desired_interval_ms()
    return 500
end

function tick(dt_ms, frame)
    local h = tonumber(os.date("%H"))
    local m = tonumber(os.date("%M"))

    draw_digit(frame,  5, math.floor(h / 10))
    draw_digit(frame, 11, h % 10)
    draw_digit(frame, 19, math.floor(m / 10))
    draw_digit(frame, 25, m % 10)

    if os.time() % 2 == 0 then
        frame:set(17, 6, BRIGHTNESS)
        frame:set(17, 2, BRIGHTNESS)
    end

    return true
end
