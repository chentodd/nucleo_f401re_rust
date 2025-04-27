## Description

A small project that controls motor in velocity and position mode:
1. `fw` contains the code for nucleo f401re development board
2. `serial_tool` contains the code for UI:
    * Connect to the board through serial port and communicate with proto messages
    * Send velocity and position commands to the board to control motor
        - Velocity commands, directly set the reference of PID velocity control loop in the board
        - Position commands, run S-curve interpolation in the board and feed interpolated velocity to PID velocity control loop
    * Display motion profile values:
        - Common, for velocity mode and position mode
          - act pos (unit: rad)
          - act vel (unit: rpm)
        - Position mode only
          - intp pos (unit: rad)
          - intp vel (unit: rad/s)
          - intp acc (unit: rad/s^2)
          - intp jerk (unit: rad/s^3)
         
here is the small [demo](https://www.youtube.com/watch?v=yFSlLiT1AuA) 
         
## Hardware

- Motor: Nidec 24H BLDC motor 8 pins
- Board: stm32f401re nucleo development board

## TODOS

- [ ] Test IMU, read IMU settings $\to$ working
- [ ] Implement and test PID auto tuning
- [ ] Add a position control loop to minimize position error between actual position and interpolated position

## Others

I'm new to Rust embedded. If you find any issues, suggestions, improvements, please feel free to open a issue directly, thanks!!

