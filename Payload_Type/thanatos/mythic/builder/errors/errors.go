// Error types for the payload build. Adds filenames and line numbers to the error message
package errors

import (
	"fmt"
	"runtime"
	"strings"
)

// The type for the build errors
type BuildError struct {
	// Line number where the error occured
	lineno int

	// Filename where the error occured
	fname string
	msg   string
}

// Construct a new build error
func New(msg string) error {
	_, fname, lineno, _ := runtime.Caller(1)
	fnameSplit := strings.Split(fname, "/")
	return &BuildError{
		msg:    msg,
		lineno: lineno,
		fname:  fnameSplit[len(fnameSplit)-1],
	}
}

// Construct a new build error with a format string
func Errorf(format string, a ...any) error {
	_, fname, lineno, _ := runtime.Caller(1)
	fnameSplit := strings.Split(fname, "/")
	return &BuildError{
		msg:    fmt.Sprintf(format, a...),
		fname:  fnameSplit[len(fnameSplit)-1],
		lineno: lineno,
	}
}

// Return the string version of the error
func (e *BuildError) Error() string {
	return fmt.Sprintf("[%s:%d]: %s", e.fname, e.lineno, e.msg)
}
