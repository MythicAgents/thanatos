import traceback
from mythic_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    CommandAttributes,
    SupportedOS,
    MythicTask,
    AgentResponse,
    ParameterType,
    MythicStatus,
    CommandParameter,
    ParameterGroupInfo,
)
from mythic_container.MythicGoRPC import (
    SendMythicRPCResponseCreate,
    MythicRPCResponseCreateMessage,
    SendMythicRPCProxyStartCommand,
    SendMythicRPCProxyStopCommand,
    MythicRPCProxyStartMessage,
    MythicRPCProxyStopMessage,
)


class SocksArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="action",
                type=ParameterType.ChooseOne,
                display_name="Action to perform.",
                description="Action to perform.",
                choices=["start", "stop"],
                default_value="start",
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
            CommandParameter(
                name="port",
                type=ParameterType.Number,
                default_value=7000,
                display_name="Port number.",
                description="Port number.",
                parameter_group_info=[ParameterGroupInfo(ui_position=2)],
            ),
        ]

    async def parse_arguments(self):
        self.load_args_from_json_string(self.command_line)


class SocksCommand(CommandBase):
    cmd = "socks"
    needs_admin = False
    help_cmd = "socks [start / stop] [port number]"
    description = "Start or stop a SOCKS 5 proxy tunneled through the agent."
    version = 1
    script_only = True
    author = "@M_alphaaa"
    argument_class = SocksArguments
    attackmapping = ["T1090"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows], builtin=True
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        try:
            action = task.args.get_arg("action")
            port = task.args.get_arg("port")

            if action == "start":
                resp = await SendMythicRPCProxyStartCommand(
                    MythicRPCProxyStartMessage(
                        TaskID=task.id, Port=port, PortType="socks5"
                    )
                )
                output = "Started socks5 server on port {}".format(port)
            else:
                resp = await SendMythicRPCProxyStopCommand(
                    MythicRPCProxyStopMessage(
                        TaskID=task.id, Port=port, PortType="socks5"
                    )
                )
                output = "Stopped socks5 server on port {}".format(port)

            if not resp.Success:
                raise Exception(resp.Error)

            else:
                await SendMythicRPCResponseCreate(
                    MythicRPCResponseCreateMessage(
                        TaskID=task.id,
                        Response=output.encode(),
                    )
                )
                task.display_params = "{} {}".format(action, port)
                task.status = MythicStatus.Completed
        except Exception as e:
            output = "".join(traceback.format_exception(e))
            output = "Error during command invocation:\n{}".format(output)

            await SendMythicRPCResponseCreate(
                MythicRPCResponseCreateMessage(TaskID=task.id, Response=output.encode())
            )

            task.set_status(MythicStatus.Error)
            task.set_stderr(output)

        return task

    async def process_response(self, response: AgentResponse):
        pass
