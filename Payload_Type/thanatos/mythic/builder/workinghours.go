// Handles parsing the working hours
package builder

import (
	"errors"
	"strconv"
	"strings"
	"time"

	thanatoserror "github.com/MythicAgents/thanatos/errors"
)

type ParsedWorkingHours struct {
	StartTime time.Duration
	EndTime   time.Duration
}

// Converts a singular working hours value '01:30' to a duration
func workingHoursValueToDuration(value string) (time.Duration, error) {
	parsedDuration := time.Duration(0)

	// Split the duration into separate hour and minute values
	stringSplit := strings.Split(value, ":")
	if len(stringSplit) == 1 {
		return parsedDuration, thanatoserror.New("did not find a ':' delimiter in the working hour time")
	} else if len(stringSplit) != 2 {
		return parsedDuration, thanatoserror.New("working hour time is malformed")
	}

	// Convert the hour portion to an integer
	hour, err := strconv.Atoi(stringSplit[0])
	if err != nil {
		return parsedDuration, thanatoserror.New("failed to parse the hours portion of the working hours")
	}

	// Validate the hour portion
	if hour > 23 {
		return parsedDuration, thanatoserror.New("hour portion cannot be greater than 23")
	} else if hour < 0 {
		return parsedDuration, thanatoserror.New("hour portion is negative")
	}

	// Convert the minute portion to an integer
	minute, err := strconv.Atoi(stringSplit[1])
	if err != nil {
		return parsedDuration, thanatoserror.New("failed to parse the minutes potion of the working hours")
	}

	// Validate the minute portion
	if minute > 60 {
		return parsedDuration, thanatoserror.New("minute portion is greater than 60")
	} else if minute < 0 {
		return parsedDuration, thanatoserror.New("minute portion is negative")
	}

	// Convert the hour period to seconds
	hour = hour * 60 * 60

	// Convert the minute period to seconds
	minute = minute * 60

	// Get the duration in total seconds
	durationSeconds := float64(hour) + float64(minute)

	// Convert the seconds to nano seconds and create a time.Duration
	parsedDuration = time.Duration(durationSeconds * float64(time.Second))

	return parsedDuration, nil
}

// Parses the working hours '00:00-23:00' format
func parseWorkingHours(workingHours string) (ParsedWorkingHours, error) {
	workingStart := time.Duration(0)
	workingEnd := time.Duration(0)

	workingHoursSplit := strings.Split(workingHours, "-")
	if len(workingHoursSplit) == 1 {
		return ParsedWorkingHours{}, thanatoserror.New("working hours value does not contain a '-' delimiter")
	}

	workingStart, err := workingHoursValueToDuration(workingHoursSplit[0])
	if err != nil {
		return ParsedWorkingHours{}, errors.Join(thanatoserror.New("failed to parse the start portion for the working hours"), err)
	}

	workingEnd, err = workingHoursValueToDuration(workingHoursSplit[1])
	if err != nil {
		return ParsedWorkingHours{}, errors.Join(thanatoserror.New("failed to parse the end portion for the working hours"), err)
	}

	if workingEnd == 0 {
		// User entered 0 for the end but probably meant 23:59. Change this to 23:59
		workingEnd = time.Duration((23 * time.Hour) + (59 * time.Second))
	}

	// Ceil and wrap the working end time
	workingEnd = (workingEnd + time.Duration(60*time.Second)) % (time.Duration(24*time.Hour) + 1)

	return ParsedWorkingHours{
		StartTime: workingStart,
		EndTime:   workingEnd,
	}, nil
}
