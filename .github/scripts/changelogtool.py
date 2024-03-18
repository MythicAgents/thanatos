#!/usr/bin/env python3

"""This script will generate changelong entries for conventional commit messages.

Conventional commit messages follow this spec:
    https://www.conventionalcommits.org/en/v1.0.0/

The changelog entry can either be generated from commit messages or extracted from the
'CHANGELOG.md' file.

Unimplemented features:
- 'BREAKING CHANGE' messages
- 'BREAKING-CHANGE' footers
- '!' prefixes
- Footers with a '#' as the separator
- Multiline footers

Extra features:
- 'Issues' footer allows referencing issue numbers
- 'Refs' footer can contain references to pull requests by prefixing the pull request number
    with a '#' (ex: #27)
- Refs can contain partial or full commit hashes
"""

from __future__ import annotations
import argparse
import sys
import subprocess
import re
import os
from enum import Enum, auto
from typing import List, Optional
from datetime import date
from dataclasses import dataclass
from pathlib import Path


HEADER = "## [{release}] - {current_date}\n\n"


class Category(Enum):
    ADDED = auto()
    CHANGED = auto()
    DEPRECATED = auto()
    FIXED = auto()
    REMOVED = auto()
    SECURITY = auto()

    def parse(value: str) -> Optional[Category]:
        match value:
            case "feat":
                return Category.ADDED
            case "fix":
                return Category.FIXED
            case "remove":
                return Category.REMOVED
            case "security":
                return Category.SECURITY
            case "deprecate":
                return Category.DEPRECATED
            case "docs" | "refactor" | "test" | "style":
                return None
            case _:
                return Category.CHANGED


@dataclass
class Issue:
    number: int
    url: str

    def __init__(self, number: int, repo: str):
        self.number = number
        self.url = repo + "/issues/" + str(number)

    def mdformat(self) -> str:
        return f"([{self.number}]({self.url}))"


@dataclass
class CommitHash:
    value: str
    url: str

    def __init__(self, hashvalue: str, repo: str):
        self.value = hashvalue
        self.url = repo + "/commit/" + hashvalue

    def mdformat(self) -> str:
        return f"([{self.value[:7]}]({self.url}))"

    def __str__(self) -> str:
        return self.value


@dataclass
class PullRequest:
    number: int
    url: str

    def __init__(self, number: int, repo: str):
        self.number = number
        self.url = repo + "/pull/" + str(number)

    def mdformat(self) -> str:
        return f"([#{self.number}]({self.url}))"


@dataclass
class CommitInfo:
    commithash: CommitHash
    category: Category
    scope: Optional[str]
    message: str
    repo: str
    issues: Optional[List[Issue]]
    commitrefs: Optional[List[CommitHash]]
    pullrefs: Optional[List[PullRequest]]

    def __init__(
        self,
        commithash: str,
        category: Category,
        scope: Optional[str],
        msg: str,
        repo: str,
    ) -> CommitInfo:
        self.commithash = CommitHash(commithash, repo)
        self.category = category
        self.scope = scope
        self.message = msg
        self.repo = repo

        refs = CommitInfo._get_refs(commithash, repo)
        if commitrefs := list(filter(lambda ref: isinstance(ref, CommitHash), refs)):
            self.commitrefs = commitrefs
        else:
            self.commitrefs = None

        if pullrefs := list(filter(lambda ref: isinstance(ref, PullRequest), refs)):
            self.pullrefs = pullrefs
        else:
            self.pullrefs = None

        self.issues = CommitInfo._get_issues(commithash, repo)

    def _get_refs(commithash: str, repo: str) -> List[CommitHash | PullRequest]:
        refscmd = (
            subprocess.check_output(
                [
                    "git",
                    "log",
                    "-n1",
                    "--format=%(trailers:key=Refs,valueonly)",
                    commithash,
                ]
            )
            .strip(b"\n")
            .decode()
        )

        reflist = list(
            map(
                lambda ref: ref.strip(),
                filter(lambda reflist: len(reflist) != 0, refscmd.split(",")),
            )
        )

        combinedrefs: List[CommitHash | PullRequest] = []
        for ref in reflist:
            if ref.startswith("#"):
                combinedrefs.append(PullRequest(int(ref[1:]), repo))
            else:
                refcommithash = (
                    subprocess.check_output(
                        [
                            "git",
                            "log",
                            "-n1",
                            "--format=%H",
                            ref,
                        ]
                    )
                    .strip(b"\n")
                    .decode()
                )

                if len(refcommithash) > 0:
                    combinedrefs.append(CommitHash(refcommithash, repo))

        return combinedrefs

    def _get_issues(commithash: str, repo_url: str) -> Optional[List[Issue]]:
        issuescmd = (
            subprocess.check_output(
                [
                    "git",
                    "log",
                    "-n1",
                    "--format=%(trailers:key=Issues,valueonly)",
                    commithash,
                ]
            )
            .strip(b"\n")
            .decode()
        )

        if issueslist := list(filter(lambda x: len(x) != 0, issuescmd.split(","))):
            return list(map(lambda n: Issue(int(n), repo_url), issueslist))

        return None

    def mdformat(self) -> str:
        output = f"- "
        if self.scope:
            output += f"**{self.scope}:** "

        output += (
            self.message + " " + f"[{str(self.commithash)[:7]}]({self.commithash.url})"
        )

        output += (
            " " + " ".join([ref.mdformat() for ref in self.commitrefs])
            if self.commitrefs
            else ""
        )

        output += (
            " " + " ".join([issue.mdformat() for issue in self.issues])
            if self.issues
            else ""
        )

        output += (
            " " + " ".join([pr.mdformat() for pr in self.pullrefs])
            if self.pullrefs
            else ""
        )

        return output


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="changelogtool.py",
        description="Manages changelog generation",
    )

    subparsers = parser.add_subparsers(dest="subcommand", help="Sub-command")

    generate = subparsers.add_parser(
        "generate",
        help="Generate a changelog entry based on Conventional Commit messages",
    )

    generate.add_argument(
        "previous",
        help="The previous release version to generate the changelog messages from",
        action="store",
    )

    generate.add_argument(
        "-n",
        "--new",
        help="The new version for the release",
        action="store",
        required=False,
    )

    generate.add_argument(
        "-r",
        "--repo",
        help="Github repository name for the changelog links (default: https://github.com/MythicAgents/thanatos)",
        default="https://github.com/MythicAgents/thanatos",
        action="store",
        required=False,
    )

    extract = subparsers.add_parser(
        "extract", help="Extract a changelog entry for a version"
    )

    extract.add_argument(
        "version",
        help="Version number for the changelog entry to extract",
        action="store",
    )

    subparsers.add_parser(
        "latest", help="Get the tag for the latest version in the changelog"
    )

    args = parser.parse_args()
    if not vars(args):
        parser.print_help()
        sys.exit(0)
    return args


def generate(previous: str, new: Optional[str], repo_url: str):
    version = previous if previous.startswith("v") else f"v{previous}"
    if new:
        if new.startswith("v"):
            release = new
        else:
            release = "v" + new

        release_tag = release.removeprefix("v")
    else:
        release = None
        release_tag = "Unreleased"

    conventional_commit_match = re.compile(
        r"^(?P<category>\w+)(\((?P<scope>\w+)\))?: (?P<message>.*)"
    )

    commitlog: List[CommitInfo] = []
    for commit in subprocess.check_output(
        ["git", "log", "--format=format:%H %s", f"{version}..HEAD"]
    ).split(b"\n"):
        commit = commit.decode()
        commithash = commit.split(" ")[0]
        commitmsg = commit[41:]

        matches = conventional_commit_match.match(commitmsg)

        if matches:
            scope = matches.group("scope")
            message = matches.group("message")

            if category := Category.parse(matches.group("category")):
                commitlog.append(
                    CommitInfo(commithash, category, scope, message, repo_url)
                )

    current_date = date.today().isoformat()
    rendered = HEADER.format(release=release_tag, current_date=current_date)

    if added := list(
        filter(lambda commit: commit.category == Category.ADDED, commitlog)
    ):
        rendered += "### Added\n\n{}\n\n".format(
            "\n".join([entry.mdformat() for entry in added])
        )

    if changed := list(
        filter(lambda commit: commit.category == Category.CHANGED, commitlog)
    ):
        rendered += "### Changed\n\n{}\n\n".format(
            "\n".join([entry.mdformat() for entry in changed])
        )

    if deprecated := list(
        filter(lambda commit: commit.category == Category.DEPRECATED, commitlog)
    ):
        rendered += "### Deprecated\n\n{}\n\n".format(
            "\n".join([entry.mdformat() for entry in deprecated])
        )

    if removed := list(
        filter(lambda commit: commit.category == Category.REMOVED, commitlog)
    ):
        rendered += "### Removed\n\n{}\n\n".format(
            "\n".join([entry.mdformat() for entry in removed])
        )

    if fixed := list(
        filter(lambda commit: commit.category == Category.FIXED, commitlog)
    ):
        rendered += "### Fixed\n\n{}\n\n".format(
            "\n".join([entry.mdformat() for entry in fixed])
        )

    if security := list(
        filter(lambda commit: commit.category == Category.SECURITY, commitlog)
    ):
        rendered += "### Security\n\n{}\n\n".format(
            "\n".join([entry.mdformat() for entry in security])
        )

    rendered = rendered.strip("\n")
    footer = f"---\n## Footer\n[{release_tag}]: {repo_url}/compare/{version}...{release if release else 'HEAD'}"
    # pprint(commitlog)
    print(f"{rendered}\n\n{footer}")


def latest():
    script_path = Path(os.path.realpath(__file__))
    changelog_path = script_path.absolute().parent.parent.parent / "CHANGELOG.md"

    version_match = re.compile(
        r"^## \[(?P<version>\d+\.\d+\.\d+)] - \d{4}-\d{2}-\d{2}$"
    )

    with open(changelog_path, "r") as f:
        for line in f:
            if matches := version_match.match(line):
                version = matches.group("version")
                print(f"v{version}")
                return

    print(f"Failed to find latest version")
    sys.exit(1)


def extract(version: str):
    version = version[1:] if version.startswith("v") else version

    script_path = Path(os.path.realpath(__file__))
    changelog_path = script_path.absolute().parent.parent.parent / "CHANGELOG.md"

    changelog_entry = ""
    with open(changelog_path, "r") as f:
        for line in f:
            if line.startswith(f"## [{version}]"):
                changelog_entry = line
                break

        for line in f:
            if line.startswith(f"## ["):
                break
            changelog_entry += line

        for line in f:
            if line.startswith(f"[{version}]: "):
                link = line.removeprefix(f"[{version}]: ").strip()
                changelog_entry = changelog_entry.replace(
                    f"## [{version}]", f"## [{version}]({link})"
                )
                break

    changelog_entry = changelog_entry.strip()

    if len(changelog_entry) == 0:
        print(f"Could not find changelog entry for version v{version}", file=sys.stderr)
        sys.exit(1)

    print(changelog_entry)


if __name__ == "__main__":
    args = parse_arguments()

    if args.subcommand == "generate":
        new_version = args.new if "new" in args else None
        generate(args.previous, new_version, args.repo)
    elif args.subcommand == "extract":
        extract(args.version)
    elif args.subcommand == "latest":
        latest()
