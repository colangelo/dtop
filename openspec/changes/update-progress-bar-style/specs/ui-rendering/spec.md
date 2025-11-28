## ADDED Requirements

### Requirement: Historical Sparkline Graphs
The system SHALL display CPU and Memory metrics as historical sparkline graphs instead of static progress bars.

#### Scenario: CPU sparkline rendering
- **WHEN** a container's CPU usage is displayed with progress bars enabled
- **THEN** the display shows a braille-based sparkline of historical CPU values
- **AND** the most recent value appears on the right side of the graph
- **AND** older values scroll to the left as new samples arrive
- **AND** the current percentage value is shown after the sparkline

#### Scenario: Memory sparkline rendering
- **WHEN** a container's memory usage is displayed with progress bars enabled
- **THEN** the display shows a braille-based sparkline of historical memory values
- **AND** the memory used/limit values are displayed after the sparkline

#### Scenario: Braille height mapping
- **WHEN** a percentage value is rendered in the sparkline
- **THEN** values 0-12.5% display as empty (`⠀`)
- **AND** values 12.5-25% display as 1-row height (`⣀`)
- **AND** values 25-50% display as 2-row height (`⣤`)
- **AND** values 50-75% display as 3-row height (`⣶`)
- **AND** values 75-100% display as 4-row height (`⣿`)

#### Scenario: History buffer behavior
- **WHEN** a new stats sample arrives for a container
- **THEN** the value is appended to the history buffer
- **AND** if the buffer exceeds its maximum size, the oldest value is removed
- **AND** the sparkline width matches the history buffer size

### Requirement: Sparkline History Size
The system SHALL maintain a history buffer sized to match the sparkline display width (approximately 20 samples).

#### Scenario: New container starts
- **WHEN** a container first starts being monitored
- **THEN** the sparkline displays only the available history (may be partially filled)
- **AND** the sparkline fills in from left to right as more samples arrive
