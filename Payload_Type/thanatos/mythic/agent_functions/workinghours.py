from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    MythicTask,
    AgentResponse,
    CommandParameter,
    ParameterType,
    ParameterGroupInfo,
)


class WorkinghoursArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="start",
                type=ParameterType.String,
                description="Start time (HH:MM)",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Set working hours",
                        required=True,
                        ui_position=1,
                    ),
                ],
            ),
            CommandParameter(
                name="end",
                type=ParameterType.String,
                description="End time (HH:MM)",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Set working hours",
                        required=True,
                        ui_position=2,
                    ),
                ],
            ),
            CommandParameter(
                name="get",
                type=ParameterType.Boolean,
                description="Get the agent's working hours",
                default_value=True,
                parameter_group_info=[
                    ParameterGroupInfo(
                        required=True,
                        group_name="Get working hours",
                    ),
                ],
            ),
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            if self.command_line[0] == "{":
                self.load_args_from_json_string(self.command_line)
            else:
                raise Exception("Invalid arguments")

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class WorkinghoursCommand(CommandBase):
    cmd = "workinghours"
    needs_admin = False
    help_cmd = "workinghours [HH:MM]-[HH:MM]"
    description = "Get or set the agent's working hours"
    version = 1
    author = "@M_alphaaa"
    argument_class = WorkinghoursArguments
    attackmapping = ["T1029"]

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        start = task.args.get_arg("start")
        end = task.args.get_arg("end")
        if start and end:
            task.display_params = f"-start {start} -end {end}"
        else:
            task.display_params = "-get"

        return task

    async def process_response(self, response: AgentResponse):
        pass
