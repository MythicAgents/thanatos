import json
import traceback
from mythic_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    CommandAttributes,
    SupportedOS,
    CommandParameter,
    ParameterGroupInfo,
    ParameterType,
    MythicTask,
    MythicStatus,
    PTTaskCompletionFunctionMessage,
    PTTaskCompletionFunctionMessageResponse,
)
from mythic_container.MythicGoRPC import (
    SendMythicRPCResponseCreate,
    MythicRPCResponseCreateMessage,
    SendMythicRPCCallbackUpdate,
    MythicRPCCallbackUpdateMessage,
)


class SleepArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="interval",
                type=ParameterType.String,
                description="Interval to sleep.",
                display_name="Interval to sleep.",
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
            CommandParameter(
                name="jitter",
                type=ParameterType.Number,
                default_value=0,
                description="Sleep jitter.",
                display_name="Sleep jitter.",
                parameter_group_info=[ParameterGroupInfo(required=False, ui_position=2)],
            ),
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            if self.command_line[0] == "{":
                self.load_args_from_json_string(self.command_line)
            else:
                args = self.command_line.split(" ")
                self.set_arg("interval", args[0])
                if len(args) > 1:
                    self.set_arg("jitter", int(args[1]))
                else:
                    self.set_arg("jitter", 0)

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


async def update_sleep_info(task: PTTaskCompletionFunctionMessage):
    await SendMythicRPCCallbackUpdate(
        MythicRPCCallbackUpdateMessage(
            CallbackID=task.TaskData.Callback.ID,
            SleepInfo=task.TaskData.Task.DisplayParams,
        )
    )


# Callback function to display output to Mythic
async def formulate_output(
    task: PTTaskCompletionFunctionMessage,
) -> PTTaskCompletionFunctionMessageResponse:
    # Get the interval and jitter from the task information
    params = json.loads(task.TaskData.Task.Params)
    interval = params["interval"]
    jitter = params["jitter"]

    # Format the output message
    output = f"Set sleep interval to {interval} seconds with a jitter of {jitter}%."

    await SendMythicRPCResponseCreate(
        MythicRPCResponseCreateMessage(
            TaskID=task.TaskData.Task.ID,
            Response=output.encode(),
        )
    )


async def post_run_actions(
    task: PTTaskCompletionFunctionMessage,
) -> PTTaskCompletionFunctionMessageResponse:
    try:
        await formulate_output(task)
        await update_sleep_info(task)
    except Exception as e:
        output = "".join(traceback.format_exception(e))
        output = f"Error during post processing:\n{output}"

        await SendMythicRPCResponseCreate(
            MythicRPCResponseCreateMessage(TaskID=task.TaskData.Task.ID, Response=output.encode())
        )

        return PTTaskCompletionFunctionMessageResponse(
            TaskID=task.TaskData.Task.ID,
            ParentTaskId=0,
            Completed=True,
            Success=False,
            Error=output,
            Stderr=output,
        )

    return PTTaskCompletionFunctionMessageResponse(
        TaskID=task.TaskData.Task.ID,
        ParentTaskId=0,
        Completed=True,
        Success=True,
    )


class SleepCommand(CommandBase):
    cmd = "sleep"
    needs_admin = False
    help_cmd = "sleep [number][suffix] [jitter]"
    description = "Change the agent's sleep interval. Suffix can either be [s, m, h]"
    version = 2
    author = "@M_alphaaa"
    argument_class = SleepArguments
    completion_functions = {"post_run_actions": post_run_actions}
    attackmapping = ["T1029"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
        builtin=True,
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        interval = task.args.get_arg("interval")
        jitter = task.args.get_arg("jitter")

        try:
            # Try to directly convert this interval to seconds
            try:
                real_interval = int(interval)
                units = "s"
            except ValueError:
                # Set the units to the last charater in the interval
                units = interval[-1]

                # Strip out the unit suffix of the interval
                interval = interval[:-1]

                if units == "s":  # Units are seconds
                    conversion_factor = 1
                elif units == "m":  # Units are minutes
                    conversion_factor = 60
                elif units == "h":  # Units are hours
                    conversion_factor = 3600
                else:
                    raise Exception("Invalid interval suffix [s, m, h]")

                # Convert the inputted interval to seconds using the conversion factor
                try:
                    real_interval = int(interval) * conversion_factor
                except ValueError:
                    raise Exception("Invalid sleep interval")

            # Check that the interval is not negative
            if real_interval < 0:
                raise Exception("Interval cannot be negative")

            # Make sure the jitter is not negative
            if jitter is not None and jitter < 0:
                raise Exception("Jitter cannot be negative")

            task.completed_callback_function = "post_run_actions"
        except Exception as e:
            output = "".join(traceback.format_exception(e))
            output = f"Error during command invocation:\n{output}"
            task.set_status(MythicStatus.Error)
            task.set_stderr(output)

        # Set the new interval
        task.args.remove_arg("interval")
        task.args.add_arg("interval", real_interval, type=ParameterType.Number)
        task.display_params = f"interval = {interval}{units}, jitter = {jitter}%"
        return task
