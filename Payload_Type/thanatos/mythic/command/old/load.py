import asyncio
import base64
import tempfile
import traceback
import hcl
import os
import json
from pathlib import Path
from shutil import copytree
from mythic_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    CommandAttributes,
    CommandParameter,
    ParameterType,
    SupportedOS,
    ParameterGroupInfo,
    MythicStatus,
    MythicTask,
    PTTaskCompletionFunctionMessage,
    PTTaskCompletionFunctionMessageResponse,
    PTRPCDynamicQueryFunctionMessage,
    PTRPCDynamicQueryFunctionMessageResponse,
)
from mythic_container.MythicGoRPC import (
    SendMythicRPCTaskUpdate,
    SendMythicRPCCommandSearch,
    MythicRPCCommandSearchMessage,
    MythicRPCTaskUpdateMessage,
    SendMythicRPCCallbackAddCommand,
    MythicRPCCallbackAddCommandMessage,
    SendMythicRPCResponseCreate,
    MythicRPCResponseCreateMessage,
)
from mythic_container.MythicRPC import MythicRPC


class LoadArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="command",
                type=ParameterType.ChooseOne,
                display_name="Command to load.",
                description="Command to load.",
                dynamic_query_function=self.get_unloaded_commands,
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            if self.command_line[0] == "{":
                self.load_args_from_dictionary(self.command_line)
            else:
                self.set_arg("command", self.command_line)

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)

    async def get_unloaded_commands(
        self, msg: PTRPCDynamicQueryFunctionMessage
    ) -> PTRPCDynamicQueryFunctionMessageResponse:
        all_commands = await SendMythicRPCCommandSearch(
            MythicRPCCommandSearchMessage(SearchPayloadTypeName="thanatos")
        )

        loaded_commands = await MythicRPC().execute(
            "get_commands", callback_id=msg.Callback, loaded_only=True
        )

        all_command_names = set([command.Name for command in all_commands.Commands])
        loaded_command_names = set(
            [command["cmd"] for command in loaded_commands.response]
        )

        unloaded_commands = sorted(all_command_names.difference(loaded_command_names))

        return PTRPCDynamicQueryFunctionMessageResponse(
            Success=True, Choices=unloaded_commands
        )


async def add_callback_command(
    task: PTTaskCompletionFunctionMessage,
) -> PTTaskCompletionFunctionMessageResponse:
    params = json.loads(task.TaskData.Task.Params)
    command_name = params["name"]

    await SendMythicRPCCallbackAddCommand(
        MythicRPCCallbackAddCommandMessage(
            TaskID=task.TaskData.Task.ID, Commands=[command_name]
        )
    )

    await SendMythicRPCResponseCreate(
        MythicRPCResponseCreateMessage(
            TaskID=task.TaskData.Task.ID,
            Response=f"Loaded {command_name} into the agent.".encode(),
        )
    )

    return PTTaskCompletionFunctionMessageResponse(
        TaskID=task.TaskData.Task.ID,
        ParentTaskId=0,
        Completed=True,
        Success=True,
    )


class LoadCommand(CommandBase):
    cmd = "load"
    needs_admin = False
    help_cmd = "load [command name]"
    description = "Load a command into the agent."
    version = 1
    author = "@M_alphaaa"
    argument_class = LoadArguments
    attackmapping = ["T1083"]
    completion_functions = {"add_callback_command": add_callback_command}
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
        builtin=True,
    )

    async def compile_command(
        self,
        taskid: int,
        command: str,
        callback_os: str,
        arch: str,
        module_config: dict,
    ) -> bytes:
        agent_build_path = tempfile.TemporaryDirectory()
        copytree(self.agent_code_path, agent_build_path.name, dirs_exist_ok=True)

        command_dir = Path(f"{agent_build_path.name}/commands/{command}")

        if callback_os == "linux":
            target = f"{arch}-unknown-linux-gnu"
        else:
            target = f"{arch}-pc-windows-gnu"

        compile_command = f"TARGET={target} make -C {str(command_dir)} build"

        await SendMythicRPCTaskUpdate(
            MythicRPCTaskUpdateMessage(
                TaskID=taskid,
                UpdateStatus="compiling command...",
            )
        )

        proc = await asyncio.create_subprocess_shell(
            compile_command,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )

        stdout, stderr = await proc.communicate()

        if proc.returncode != 0:
            output = "Failed to compile command:\n"
            output += "[stdout]:\n{}\n".format(stdout)
            output += "[stderr]:\n{}".format(stderr)
            raise Exception(output)

        out_path = command_dir.joinpath(
            Path(module_config["command"][command][callback_os]["path"])
        )

        with open(out_path, "rb") as f:
            command_data = f.read()

        if not Path("/Mythic/built-commands").exists():
            os.mkdir("/Mythic/built-commands")

        with open(f"/Mythic/built-commands/{command}_{callback_os}_{arch}", "wb") as f:
            f.write(command_data)

        return command_data

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        command = task.args.get_arg("command")
        try:
            arch = (
                "x86_64"
                if "x86_64" == task.callback.__dict__["architecture"]
                else "i686"
            )

            callback_os = (
                SupportedOS.Linux
                if "Linux" in task.callback.__dict__["os"]
                else SupportedOS.Window
            ).lower()

            command_dir = Path(f"{self.agent_code_path}/commands/{command}")

            with open(f"{str(command_dir)}/module.hcl") as f:
                module_config = hcl.load(f)

            if Path(f"/Mythic/built-commands/{command}_{callback_os}_{arch}").exists():
                with open(
                    f"/Mythic/built-commands/{command}_{callback_os}_{arch}", "rb"
                ) as f:
                    data = f.read()
            else:
                data = await self.compile_command(
                    task.id, command, callback_os, arch, module_config
                )

            b64data = base64.b64encode(data).decode()
        except Exception as e:
            output = "".join(traceback.format_exception(e))
            output = "Error during command invocation:\n{}".format(output)
            task.set_status(MythicStatus.Error)
            task.set_stderr(output)
            task.completed = True
            return task

        task.display_params = command
        task.args.remove_arg("command")
        task.args.add_arg("name", command, type=ParameterType.String)
        task.args.add_arg("b64data", b64data, type=ParameterType.String)

        task.args.add_arg("force_internal", False, type=ParameterType.Boolean)
        task.args.add_arg("run_detached", True, type=ParameterType.Boolean)

        if "internal" in module_config["command"][command][callback_os]:
            internal = module_config["command"][command][callback_os]["internal"]

            if isinstance(internal, dict):
                if "force" in internal:
                    task.args.set_arg("force_internal", internal["force"])
                if "detached" in internal:
                    task.args.set_arg("run_detached", internal["detached"])
            elif isinstance(internal, bool):
                task.args.set_arg("force_internal", internal)

        task.completed_callback_function = "add_callback_command"
        return task
