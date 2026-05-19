local phase = 0.0

function desired_interval_ms()
    return 50
end

function tick(dt_ms, frame)
    -- 2-second full cycle
    phase = phase + (dt_ms / 2000.0) * (math.pi * 2)
    local brightness = math.floor(30 + 225 * (math.sin(phase) + 1) / 2)
    frame:fill_rect(0, 0, frame.ROWS, frame.COLS, brightness)
    return true
end
