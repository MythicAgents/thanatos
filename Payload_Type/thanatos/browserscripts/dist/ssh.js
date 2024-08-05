"use strict";
function ssh(task, responses) {
    if (task.status.includes("error")) {
        var combined = responses.reduce(function (prev, cur) {
            return prev + cur;
        }, "");
        return { "plaintext": combined };
    }
    if (!(responses.length > 0)) {
        return { "plaintext": "No response yet from agent..." };
    }
    var task_params = {};
    try {
        task_params = JSON.parse(task.original_params);
    }
    catch (error) {
        console.log(error);
        var combined = responses.reduce(function (prev, cur) {
            return prev + cur;
        }, "");
        return { "plaintext": combined };
    }
    if (task_params.hasOwnProperty("list")) {
        var fileFormats = [
            {
                hoverText: "Archive File",
                icon: "archive",
                color: "orange",
                extensions: [
                    "a", "ar", "cpio", "shar", "LBR", "lbr", "mar", "sbx", "tar", "bz2",
                    "F", "gz", "lz", "lz4", "lzma", "lzo", "rz", "sfark", "sz", "?Q?", "?Z?", "xz",
                    "z", "Z", "zst", "??", "7z", "s7z", "ace", "afa", "alz", "apk", "arc", "arc",
                    "arj", "b1", "b6z", "ba", "bh", "cab", "car", "cfs", "cpt", "dar", "dd", "dgc",
                    "ear", "gca", "ha", "hki", "ice", "jar", "kgb", "lzh", "lzx", "pak", "pak",
                    "parti", "paq6", "pea", "pim", "pit", "qda", "rar", "rk", "sda", "sea", "sen",
                    "sfx", "shk", "sit", "sitx", "sqx", "tar", "tbz2", "uc", "uca", "uha", "war",
                    "wim", "xar", "xp3", "yz1", "zip", "zoo", "zpaq", "zz", "ecc", "ecsbx", "par",
                    "par2", "rev"
                ],
            },
            {
                hoverText: "Disk Image",
                icon: "diskimage",
                color: "goldenred",
                extensions: ["dmg", "iso", "vmdk"],
            },
            {
                hoverText: "Microsoft Word Document",
                icon: "word",
                color: "cornflowerblue",
                extensions: ["doc", "docx", "dotm", "dot", "wbk", "docm", "dotx", "docb"],
            },
            {
                hoverText: "Microsoft Excel Document",
                icon: "excel",
                color: "darkseagreen",
                extensions: ["csv", "xls", "xlsx", "xlsm", "xltx", "xltm", "xlmx", "xlmt"],
            },
            {
                hoverText: "Portable Document Format",
                icon: "pdf",
                color: "indianred",
                extensions: ["pdf"],
            },
            {
                hoverText: "Database File Format",
                icon: "database",
                color: "burlywood",
                extensions: [".db", ".sql", ".psql", "sqlite3"],
            },
            {
                hoverText: "Key Credential File",
                icon: "key",
                color: "gold",
                extensions: [".pem", ".ppk", ".cer", ".pvk", ".pfx"],
            },
            {
                hoverText: "Source Code",
                icon: "code",
                color: "dodgerblue",
                extensions: [
                    ".config", ".ps1", ".psm1", ".psd1", ".vbs", ".js", ".py", ".pl", ".rb", ".go",
                    ".xml", ".html", ".css", ".sh", ".bash", ".yaml", ".yml", ".c", ".cpp", ".h",
                    ".hpp", ".cs", ".sln", ".csproj", "toml", "gitignore", "rs"
                ],
            },
            {
                hoverText: "Image File",
                icon: "image",
                color: "paleturquoise",
                extensions: [
                    ".2000", ".ani", ".anim", ".apng", ".art", ".avif", ".bmp", ".bpg", ".bsave",
                    ".cal", ".cin", ".cpc", ".cpt", ".cur", ".dds", ".dpx", ".ecw", ".ep", ".exr",
                    ".fits", ".flic", ".flif", ".fpx", ".gif", ".hdr", ".hdri", ".hevc", ".icer",
                    ".icns", ".ico", ".ics", ".ilbm", ".it", ".jbig", ".jbig2", ".jng", ".jpeg",
                    ".jpeg", ".jpeg", ".jpeg", ".jpeg", ".jpeg", ".jpeg", ".jpeg", ".kra", ".logluv",
                    ".ls", ".miff", ".mng", ".nrrd", ".pam", ".pbm", ".pcx", ".pgf", ".pgm", ".pictor",
                    ".png", ".pnm", ".ppm", ".psb", ".psd", ".psp", ".qtvr", ".ras", ".rgbe", ".sgi",
                    ".tga", ".tiff", ".tiff", ".tiff", ".tiff", ".ufo", ".ufp", ".wbmp", ".webp",
                    ".xbm", ".xcf", ".xl", ".xpm", ".xr", ".xs", ".xt", ".xwd"
                ],
            }
        ];
        var data = {};
        try {
            data = JSON.parse(responses[0]);
        }
        catch (error) {
            console.log(error);
            var combined = responses.reduce(function (prev, curr) {
                return prev + curr;
            }, "");
            return { "plaintext": combined };
        }
        var headers = [
            { "plaintext": "actions", "type": "button", "width": 100 },
            { "plaintext": "name", "type": "string", "fillWidth": true },
            { "plaintext": "size", "type": "size", "width": 150 },
            { "plaintext": "uid", "type": "string", "width": 125 },
            { "plaintext": "gid", "type": "string", "width": 125 },
            { "plaintext": "permissions", "type": "string", "width": 150 },
            { "plaintext": "last modified", "type": "string", "fillWidth": true },
            { "plaintext": "metadata", "type": "button", "width": 100 }
        ];
        var rows = [];
        var _loop_1 = function (file) {
            var dateOptions = {
                weekday: "long",
                year: "numeric",
                month: "long",
                day: "numeric",
                hour: "numeric",
                minute: "numeric",
                second: "numeric",
                timeZone: Intl.DateTimeFormat().resolvedOptions().timeZone,
                timeZoneName: "long"
            };
            var date = new Date(file["access_time"]);
            var accessed = new Intl.DateTimeFormat(navigator.language, dateOptions).format(date);
            date = new Date(file["modify_time"]);
            var modified = new Intl.DateTimeFormat(navigator.language, dateOptions).format(date);
            dateOptions.weekday = "short";
            dateOptions.month = "short";
            dateOptions.timeZoneName = "short";
            var modifiedShort = new Intl.DateTimeFormat(navigator.language, dateOptions).format(date);
            var fileIcon = "";
            var fileHoverText = "";
            var fileColor = "";
            if (file["is_file"]) {
                var fileExtension_1 = file["name"].split(".").slice(-1)[0].toLowerCase();
                var fileFormat = fileFormats.find(function (entry) { return entry.extensions.includes(fileExtension_1); });
                if (fileFormat !== undefined) {
                    fileIcon = fileFormat.icon;
                    fileHoverText = fileFormat.hoverText;
                    fileColor = fileFormat.color;
                }
                else {
                    fileIcon = "file";
                    fileHoverText = "File";
                    fileColor = "white";
                }
            }
            else {
                fileIcon = "openfolder";
                fileHoverText = "Directory";
                fileColor = "yellow";
            }
            var ssh_params = {
                credentials: task_params["credentials"],
                agent: task_params["agent"],
                host: task_params["host"],
                port: task_params["port"],
            };
            var row = {
                "actions": {
                    "button": {
                        "name": "task",
                        "startIcon": "list",
                        "type": "menu",
                        "value": [
                            {
                                "name": "Cat",
                                "type": "task",
                                "disabled": !file["is_file"],
                                "ui_feature": "ssh",
                                "parameters": JSON.stringify(Object.assign({}, ssh_params, { cat: file["full_name"] })),
                            },
                            {
                                "name": "Delete",
                                "type": "task",
                                "ui_feature": "ssh",
                                "parameters": JSON.stringify(Object.assign({}, ssh_params, { rm: file["full_name"] })),
                                "startIcon": "delete"
                            },
                            {
                                "name": "Download",
                                "type": "task",
                                "disabled": !file["is_file"],
                                "ui_feature": "ssh",
                                "parameters": JSON.stringify(Object.assign({}, ssh_params, { download: file["full_name"] })),
                                "startIcon": "download"
                            },
                            {
                                "name": "List",
                                "type": "task",
                                "ui_feature": "ssh",
                                "disabled": file["is_file"],
                                "parameters": JSON.stringify(Object.assign({}, ssh_params, { list: file["full_name"] })),
                                "startIcon": "list"
                            },
                        ]
                    }
                },
                "name": {
                    "plaintext": file["name"],
                    "startIcon": fileIcon,
                    "startIconHoverText": fileHoverText,
                    "startIconColor": fileColor,
                    "copyIcon": true
                },
                "size": {
                    "plaintext": file["size"]
                },
                "last modified": {
                    "plaintext": modifiedShort
                },
            };
            var metadataInfo = {
                "Last accessed": accessed,
                "Last modified": modified,
                "Owner UID": file["permissions"]["uid"].toString(),
                "Owner GID": file["permissions"]["gid"].toString(),
                "Permissions": file["permissions"]["permissions"]
            };
            row["metadata"] = {
                "button": {
                    "name": "view",
                    "type": "dictionary",
                    "title": "Metadata for ".concat(file["name"]),
                    "leftColumnTitle": "Attribute",
                    "rightColumnTitle": "Value",
                    "value": metadataInfo,
                    "startIcon": "list",
                    "hoverText": "View ".concat(file["is_file"] ? "file" : "directory", " metadata")
                }
            };
            row["uid"] = {
                "plaintext": file["permissions"]["uid"].toString(),
            };
            row["gid"] = {
                "plaintext": file["permissions"]["gid"].toString(),
            };
            row["permissions"] = {
                "plaintext": file["permissions"]["permissions"]
            };
            rows.push(row);
        };
        for (var _i = 0, _a = data["files"]; _i < _a.length; _i++) {
            var file = _a[_i];
            _loop_1(file);
        }
        return {
            "table": [
                {
                    "headers": headers,
                    "rows": rows
                }
            ]
        };
    }
    else if (task_params.hasOwnProperty("download")) {
        try {
            return {
                "download": [{
                        "agent_file_id": responses[0],
                        "variant": "contained",
                        "name": "Download " + task["display_params"]
                    }]
            };
        }
        catch (error) {
            var combined = responses.reduce(function (prev, curr) {
                return prev + curr;
            }, "");
            return { 'plaintext': combined };
        }
    }
    else {
        return {
            "plaintext": responses.reduce(function (prev, curr) {
                return prev + "\n" + curr;
            })
        };
    }
}
