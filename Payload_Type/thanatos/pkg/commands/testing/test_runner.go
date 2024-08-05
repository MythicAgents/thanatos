package testing

import (
	"crypto/rand"
	"encoding/json"
	"math"
	"math/big"
	"reflect"
	"testing"

	"github.com/MythicAgents/thanatos/commands/utils"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

func RunTestCases(
	t *testing.T,
	command agentstructs.Command,
	commandTaskFns utils.CommandTaskFunction,
	commandPostRunFns utils.CommandPostRunFunction,
	testCases []TestCase,
) {
	for _, testCase := range testCases {
		t.Run(testCase.Name, func(t *testing.T) {
			inputTask := createTaskData(t, command)

			if !runPreTaskingTests(t, &inputTask, command, commandTaskFns, testCase) {
				return
			}

			if commandPostRunFns != nil {
				runResponseProcessingTests(t, &inputTask, command, commandPostRunFns, testCase)
			}
		})
	}
}

func createTaskData(t *testing.T, command agentstructs.Command) agentstructs.PTTaskMessageAllData {
	randTaskID, err := rand.Int(rand.Reader, big.NewInt(math.MaxInt32))
	if err != nil {
		t.Fatalf("rand error: %s", err)
	}

	taskId := int(randTaskID.Int64())
	taskMessageData := agentstructs.PTTaskMessageTaskData{
		ID: taskId,
	}

	taskData := agentstructs.PTTaskMessageAllData{
		Task: taskMessageData,
	}

	for _, param := range command.CommandParameters {
		if err := taskData.Args.AddArg(param); err != nil {
			t.Fatalf("failed to add parameter '%s': %s", param.Name, err.Error())
		}
	}

	return taskData
}

func (c TestCase) String() string {
	testCaseJson, _ := json.MarshalIndent(c, "", "  ")
	return string(testCaseJson)
}

func runPreTaskingTests(
	t *testing.T,
	inputTask *agentstructs.PTTaskMessageAllData,
	command agentstructs.Command,
	commandTaskFns utils.CommandTaskFunction,
	testCase TestCase,
) bool {
	if !runParseArgsTest(t, inputTask, command, commandTaskFns, testCase) {
		return false
	}

	return runCreateTaskingTest(t, inputTask, command, commandTaskFns, testCase)
}

func runParseArgsTest(
	t *testing.T,
	inputTask *agentstructs.PTTaskMessageAllData,
	command agentstructs.Command,
	commandTaskFns utils.CommandTaskFunction,
	testCase TestCase,
) bool {
	// Run the ParseArgString function
	err := commandTaskFns.ParseArgString(&inputTask.Args, testCase.InputCLI)

	// Function is expected to fail
	if !testCase.PreTasking.ParseArgsResult.Success {
		// The function did not fail
		if err == nil {
			t.Logf("result: %+v", err)
			t.Logf("test function: '%s.ParseArgString()'", reflect.TypeOf(commandTaskFns).Name())
			t.Fatal("function returned with no errors but it was expected to fail")
		}
		return false
	}

	// Function is expected to be successful but returned an error
	if testCase.PreTasking.ParseArgsResult.Success && err != nil {
		t.Logf("result: %+v", err)
		t.Logf("test function: '%s.ParseArgString()'", reflect.TypeOf(commandTaskFns).Name())
		t.Fatal("function returned an error but was expected to succeed")
	}

	// Check each expected parameter value
	for _, expectedValue := range testCase.PreTasking.ParseArgsResult.ArgValues {
		value, err := inputTask.Args.GetArg(expectedValue.Name)

		if err != nil {
			t.Logf("test function: '%s.ParseArgString()'", reflect.TypeOf(commandTaskFns).Name())
			t.Fatalf("could not get resulting arg value with name '%s': %s", expectedValue.Name, err.Error())
		}

		if value != expectedValue.Value {
			t.Logf("result: %v", value)
			t.Logf("test function: '%s.ParseArgString()'", reflect.TypeOf(commandTaskFns).Name())
			t.Fatalf("resulting '%s' argument value is '%v' which does not equal expected value of '%v'", expectedValue.Name, value, expectedValue.Value)
		}
	}

	return true
}

func runCreateTaskingTest(
	t *testing.T,
	inputTask *agentstructs.PTTaskMessageAllData,
	command agentstructs.Command,
	commandTaskFns utils.CommandTaskFunction,
	testCase TestCase,
) bool {
	// Run the CreateTasking function
	createTaskingReturn := commandTaskFns.CreateTasking(inputTask)

	if createTaskingReturn.TaskID != inputTask.Task.ID {
		t.Logf("test function: '%s.CreateTasking()'", reflect.TypeOf(commandTaskFns).Name())
		t.Fatalf("input task ID of '%d' does not match returned task ID of '%d'", inputTask.Task.ID, createTaskingReturn.TaskID)
	}

	// CreateTasking expected a failure
	if !testCase.PreTasking.CreateTaskingResult.Success {
		// Function succeeded
		if createTaskingReturn.Success {
			t.Logf("[task.Success]: %t", createTaskingReturn.Success)
			t.Logf("[task.Error]: %s", createTaskingReturn.Error)
			t.Logf("[task.Stdout]: %v", createTaskingReturn.Stdout)
			t.Logf("[task.Stderr]: %v", createTaskingReturn.Stderr)
			t.Logf("test function: '%s.CreateTasking()'", reflect.TypeOf(commandTaskFns).Name())
			t.Fatalf("function returned a failure but was expected to succeed")
		}

		return false
	}

	// Function returned a failure but was expected to succeed
	if !createTaskingReturn.Success && testCase.PreTasking.CreateTaskingResult.Success {
		t.Logf("[task.Success]: %t", createTaskingReturn.Success)
		t.Logf("[task.Error]: %s", createTaskingReturn.Error)
		t.Logf("[task.Stdout]: %v", createTaskingReturn.Stdout)
		t.Logf("[task.Stderr]: %v", createTaskingReturn.Stderr)
		t.Logf("test function: '%s.CreateTasking()'", reflect.TypeOf(commandTaskFns).Name())
		t.Fatalf("function returned a failure but was expected to succeed")
	}

	// Check each expected parameter value
	for _, expectedValue := range testCase.PreTasking.CreateTaskingResult.ArgValues {
		value, err := inputTask.Args.GetArg(expectedValue.Name)

		if err != nil {
			t.Logf("test function: '%s.CreateTasking()'", reflect.TypeOf(commandTaskFns).Name())
			t.Fatalf("could not get resulting arg value with name '%s': %s", expectedValue.Name, err.Error())
		}

		if value != expectedValue.Value {
			t.Logf("result: %v", value)
			t.Logf("test function: '%s.CreateTasking()'", reflect.TypeOf(commandTaskFns).Name())
			t.Fatalf("resulting '%s' argument value is '%T(%v)' which does not equal expected value of '%T(%v)'", expectedValue.Name, value, value, expectedValue.Value, expectedValue.Value)
		}
	}

	return true
}

func runResponseProcessingTests(
	t *testing.T,
	inputTask *agentstructs.PTTaskMessageAllData,
	command agentstructs.Command,
	commandPostRunFns utils.CommandPostRunFunction,
	testCase TestCase,
) {
	if !runCompletionFunctionTest(t, inputTask, command, commandPostRunFns, testCase) {
		return
	}
}

func runCompletionFunctionTest(
	t *testing.T,
	inputTask *agentstructs.PTTaskMessageAllData,
	command agentstructs.Command,
	commandPostRunFns utils.CommandPostRunFunction,
	testCase TestCase,
) bool {
	// Run the PostRunAction function
	postRunReturn := commandPostRunFns.PostRunAction(inputTask, nil, nil)

	if postRunReturn.TaskID != inputTask.Task.ID {
		t.Logf("test function: '%s.PostRunAction()'", reflect.TypeOf(commandPostRunFns).Name())
		t.Fatalf("input task ID of '%d' does not match returned task ID of '%d'", inputTask.Task.ID, postRunReturn.TaskID)
	}

	// Expected a failure
	if !testCase.ResponseProcessing.CompletionFunctionResult.Success {
		// Function succeeded
		if postRunReturn.Success {
			t.Logf("[msg.Success]: %t", postRunReturn.Success)
			t.Logf("[msg.Error]: %s", postRunReturn.Error)
			t.Logf("[msg.Stdout]: %v", postRunReturn.Stdout)
			t.Logf("[msg.Stderr]: %v", postRunReturn.Stderr)
			t.Logf("test function: '%s.PostRunAction()'", reflect.TypeOf(commandPostRunFns).Name())
			t.Fatalf("function returned a failure but was expected to succeed")
		}

		return false
	}

	// Function returned a failure but was expected to succeed
	if !postRunReturn.Success && testCase.ResponseProcessing.CompletionFunctionResult.Success {
		t.Logf("[msg.Success]: %t", postRunReturn.Success)
		t.Logf("[msg.Error]: %s", postRunReturn.Error)
		t.Logf("[msg.Stdout]: %v", postRunReturn.Stdout)
		t.Logf("[msg.Stderr]: %v", postRunReturn.Stderr)
		t.Logf("test function: '%s.PostRunAction()'", reflect.TypeOf(commandPostRunFns).Name())
		t.Fatalf("function returned a failure but was expected to succeed")
	}

	return true
}
