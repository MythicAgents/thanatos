from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    MythicTask,
    AgentResponse,
    CommandParameter,
    ParameterType,
    ParameterGroupInfo,
)


class SleepArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="interval",
                type=ParameterType.String,
                description="Interval to sleep.",
            ),
            CommandParameter(
                name="jitter",
                type=ParameterType.Number,
                default_value=0,
                description="Sleep jitter.",
                parameter_group_info=[ParameterGroupInfo(required=False)],
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


class SleepCommand(CommandBase):
    cmd = "sleep"
    needs_admin = False
    help_cmd = "sleep [number][suffix] [jitter]"
    description = "Change the agent's sleep interval. Suffix can either be [s, m, h]"
    version = 1
    author = "@M_alphaaa"
    argument_class = SleepArguments
    attackmapping = ["T1029"]

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        interval = task.args.get_arg("interval")
        jitter = task.args.get_arg("jitter")

        # Try to directly convert this interval to seconds
        try:
            real_interval = int(interval)
            units = "s"
        except Exception:
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
            except Exception:
                raise Exception("Invalid interval")

        # Check that the interval is not negative
        if real_interval < 0:
            raise Exception("Interval cannot be negative")

        # Make sure the jitter is not negative
        if jitter is not None and jitter < 0:
            raise Exception("Jitter cannot be negative")

        # Set the new interval
        task.args.set_arg("interval", real_interval)
        task.display_params = f"interval = {interval}{units}, jitter = {jitter}%"

        return task

    async def process_response(self, response: AgentResponse):
        pass
