// Error type which adds filenames and line numbers to the error message
package errors

import (
	"fmt"
	"runtime"
	"strings"
)

// The type for the errors
type Error struct {
	// Runtime context
	line     int
	file     string
	function string

	// Error message
	msg string
}

// Construct a new error
func New(msg string) error {
	ev := Error{
		msg:  msg,
		line: 0,
	}

	pc, file, line, ok := runtime.Caller(1)
	if !ok {
		return &ev
	}

	ev.line = line

	funcInfo := runtime.FuncForPC(pc)
	if funcInfo != nil {
		ev.function = funcInfo.Name()
	}

	filenameSplit := strings.Split(file, "/")
	ev.file = filenameSplit[len(filenameSplit)-1]

	return &ev
}

// Construct a new error with a format string
func Errorf(format string, a ...any) error {
	ev := Error{
		msg:  fmt.Sprintf(format, a...),
		line: 0,
	}

	pc, file, line, ok := runtime.Caller(1)
	if !ok {
		return &ev
	}

	ev.line = line

	funcInfo := runtime.FuncForPC(pc)
	if funcInfo != nil {
		ev.function = funcInfo.Name()
	}

	fnameSplit := strings.Split(file, "/")
	ev.file = fnameSplit[len(fnameSplit)-1]

	return &ev
}

// Return the string version of the error
func (e *Error) Error() string {
	errorString := ""
	if e.line > 0 && (e.file != "" || e.function != "") {
		errorString += "["

		if e.file != "" {
			errorString += fmt.Sprintf("%s", e.file)
		} else {
			errorString += "<unknown>"
		}

		if e.line > 0 {
			errorString += fmt.Sprintf(":%d", e.line)
		} else {
			errorString += ":<unknown>"
		}

		if e.function != "" {
			errorString += fmt.Sprintf(" %s", e.function)
		} else {
			errorString += " <unknown>"
		}

		errorString += "]"

	} else {
		errorString += "[<unknown>]"
	}

	errorString += fmt.Sprintf(" %s", e.msg)
	return errorString
}
