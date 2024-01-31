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
    PTTaskCompletionFunctionMessage,
    PTTaskCompletionFunctionMessageResponse,
)
from mythic_container.MythicGoRPC import (
    SendMythicRPCResponseCreate,
    MythicRPCResponseCreateMessage,
)


class WorkingHoursArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="start",
                type=ParameterType.String,
                description="Start of the working hours (HH:MM).",
                display_name="Start of the working hours (HH:MM).",
                default_value="00:00",
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
            CommandParameter(
                name="end",
                type=ParameterType.String,
                description="End of the working hours (HH:MM).",
                display_name="End of the working hours (HH:MM).",
                default_value="23:59",
                parameter_group_info=[ParameterGroupInfo(ui_position=2)],
            ),
        ]

    async def parse_arguments(self):
        if len(self.command_line) > 0:
            if self.tasking_location == "command_line":
                if self.command_line[0] == "{":
                    self.load_args_from_json_string(self.command_line)
                else:
                    args = self.command_line.split(" ")
                    try:
                        start = args[0]
                        end = args[1]
                    except Exception:
                        raise Exception(
                            "Working hours not provided (usage: workinghours HH:MM HH:MM"
                        )

                    self.set_arg("start", start)
                    self.set_arg("end", end)

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


def format_time(hour: int, minute: int) -> str:
    hour_12 = hour if hour < 12 else hour - 12
    hour_12 = hour_12 if hour_12 != 0 else 12
    tod = "am" if hour < 12 else "pm"
    return f"{hour_12}:{minute:02d}{tod} ({hour:02d}:{minute:02d})"


# Callback function to display output to Mythic
async def formulate_output(task: PTTaskCompletionFunctionMessage):
    params = json.loads(task.TaskData.Task.Params)
    working_start_hours = int(int(params["start"]) / 3600)
    working_start_minutes = int(((params["start"] / 3600) - working_start_hours) * 60)

    working_start_hours, working_start_minutes = int(working_start_hours), int(
        working_start_minutes
    )

    start_time_format = format_time(working_start_hours, working_start_minutes)

    working_end_hours = int(int(params["end"]) / 3600)
    working_end_minutes = int(((params["end"] / 3600) - working_end_hours) * 60)

    end_time_format = format_time(working_end_hours, working_end_minutes)

    output = (
        f"Agent will only beacon between {start_time_format} and"
        f" {end_time_format} following the host's system time."
    )

    await SendMythicRPCResponseCreate(
        MythicRPCResponseCreateMessage(
            TaskID=task.TaskData.Task.ID,
            Response=output.encode(),
        )
    )


# Callback function to display output to Mythic
async def post_run_actions(
    task: PTTaskCompletionFunctionMessage,
) -> PTTaskCompletionFunctionMessageResponse:
    try:
        await formulate_output(task)
    except Exception as e:
        output = "".join(traceback.format_exception(e))
        output = f"Error during post processing:\n{output}"

        await SendMythicRPCResponseCreate(
            MythicRPCResponseCreateMessage(
                TaskID=task.TaskData.Task.ID, Response=output.encode()
            )
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


class WorkingHoursCommand(CommandBase):
    cmd = "workinghours"
    needs_admin = False
    help_cmd = "workinghours HH:MM HH:MM"
    description = "Set the agent's working hours"
    version = 2
    author = "@M_alphaaa"
    argument_class = WorkingHoursArguments
    completion_functions = {"post_run_actions": post_run_actions}
    attackmapping = ["T1029"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
        builtin=True,
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        working_start = task.args.get_arg("start")
        working_end = task.args.get_arg("end")

        # Parse the start portion of the working hours
        working_start = working_start.split(":")
        try:
            working_start_hours = int(working_start[0])
        except IndexError:
            raise Exception("Hour portion of the start working hours not provided")
        except ValueError:
            raise Exception("Hour portion of the start working hours is not an integer")

        try:
            working_start_minutes = int(working_start[1])
        except IndexError:
            raise Exception("Minute portion of the start working hours not provided")
        except ValueError:
            raise Exception(
                "Minute portion of the start working hours is not an integer"
            )

        working_start = (int(working_start_hours) * 3600) + (
            int(working_start_minutes) * 60
        )
        # Parse the end portion of the working hours
        working_end = working_end.split(":")
        try:
            working_end_hours = int(working_end[0])
        except IndexError:
            raise Exception("Hour portion of the end working hours not provided")
        except ValueError:
            raise Exception("Hour portion of the end working hours is not an integer")

        try:
            working_end_minutes = int(working_end[1])
        except IndexError:
            raise Exception("Minute portion of the end working hours not provided")
        except ValueError:
            raise Exception("Minute portion of the end working hours is not an integer")

        working_end = (
            (int(working_end_hours) * 3600) + (int(working_end_minutes) * 60) + 60
        )

        if working_start >= working_end:
            raise Exception(
                "Invalid working hours. Start time is greater than or equal to end time"
            )

        # Add the start portion of the working hours as an integer
        task.args.remove_arg("start")
        task.args.add_arg("start", working_start, type=ParameterType.Number)

        task.args.remove_arg("end")
        task.args.add_arg("end", working_end, type=ParameterType.Number)

        task.completed_callback_function = "post_run_actions"

        task.display_params = (
            f"start = {working_start_hours:02d}:{working_start_minutes:02d}, "
            f"end = {working_end_hours:02d}:{working_end_minutes:02d}"
        )
        return task
