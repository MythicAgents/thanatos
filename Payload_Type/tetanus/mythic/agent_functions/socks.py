from mythic_payloadtype_container.MythicCommandBase import *
from mythic_payloadtype_container.MythicRPC import *


class SocksArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="action",
                type=ParameterType.ChooseOne,
                choices=["start", "stop"],
                default_value="start",
                description="Start or Stop socks through this callback.",
                parameter_group_info=[
                    ParameterGroupInfo(
                        ui_position=1
                    )
                ]
            ),
            CommandParameter(
                name="port",
                type=ParameterType.Number,
                description="Port to open on Mythic for socks5 proxying",
                parameter_group_info=[
                    ParameterGroupInfo(
                        ui_position=2
                    )
                ]
            ),
        ]

    async def parse_arguments(self):
        self.load_args_from_json_string(self.command_line)


class SocksCommand(CommandBase):
    cmd = "socks"
    needs_admin = False
    help_cmd = "socks"
    description = "Start or stop a socks5 instance."
    version = 1
    author = "@M_alphaaa"
    argument_class = SocksArguments
    attackmapping = ["T1572"]

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        if task.args.get_arg("action") == "start":
            resp = await MythicRPC().execute("control_socks", task_id=task.id, start=True, port=task.args.get_arg("port"))
            if resp.status != MythicStatus.Success:
                task.status = MythicStatus.Error
                raise Exception(resp.error)
        else:
            resp = await MythicRPC().execute("control_socks", task_id=task.id, stop=True, port=task.args.get_arg("port"))
            if resp.status != MythicStatus.Success:
                task.status = MythicStatus.Error
                raise Exception(resp.error)
        return task

    async def process_response(self, response: AgentResponse):
        pass
