function ls(task, responses) {
  if (task.status.includes("error")) {
    const combined = responses.reduce((prev, cur) => {
      return prev + cur;
    }, "");
    return { 'plaintext': combined };
  }

  if (!(responses.length > 0)) {
    return { "plaintext": "No response yet from agent..." }
  }

  const fileFormats = [
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


  let data = {};
  try {
    data = JSON.parse(responses[0]);
  } catch (error) {
    console.log(error);
    const combined = responses.reduce((prev, curr) => {
      return prev + curr;
    }, "");

    return { "plaintext": combined }
  }

  let headers = [
    { "plaintext": "actions", "type": "button", "width": 100 },
    { "plaintext": "name", "type": "string", "fillWidth": true },
    { "plaintext": "size", "type": "size", "width": 150 },
  ];

  let platform = data["platform"];

  if (platform == "Windows") {
    headers.push({ "plaintext": "owner", "type": "string", "fillWidth": true, });
  } else if (platform == "Linux") {
    headers.push({ "plaintext": "user", "type": "string", "width": 125 });
    headers.push({ "plaintext": "group", "type": "string", "width": 125 });
    headers.push({ "plaintext": "permissions", "type": "string", "width": 150 });
  }

  headers.push({ "plaintext": "last modified", "type": "string", "fillWidth": true });
  headers.push({ "plaintext": "metadata", "type": "button", "width": 100 });

  let rows = [];
  for (const file of data["files"]) {
    let dateOptions = {
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

    let date = new Date(file["permissions"]["creation_date"] * 1000);
    let created = new Intl.DateTimeFormat(navigator.language, dateOptions).format(date);

    date = new Date(file["access_time"]);
    let accessed = new Intl.DateTimeFormat(navigator.language, dateOptions).format(date);

    date = new Date(file["modify_time"]);
    let modified = new Intl.DateTimeFormat(navigator.language, dateOptions).format(date);

    dateOptions.weekday = "short"
    dateOptions.month = "short"
    dateOptions.timeZoneName = "short"
    let modifiedShort = new Intl.DateTimeFormat(navigator.language, dateOptions).format(date);

    let fileIcon = "";
    let fileHoverText = "";
    let fileColor = "";

    if (file["is_file"]) {
      let fileExtension = file["name"].split(".").slice(-1)[0].toLowerCase()
      let fileFormat = fileFormats.find((entry) => entry.extensions.includes(fileExtension));

      if (fileFormat !== undefined) {
        fileIcon = fileFormat.icon
        fileHoverText = fileFormat.hoverText
        fileColor = fileFormat.color
      } else {
        fileIcon = "file";
        fileHoverText = "File";
        fileColor = "white";
      }

    } else {
      fileIcon = "openfolder";
      fileHoverText = "Directory";
      fileColor = "yellow";
    }

    let row = {
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
              "ui_feature": "cat",
              "parameters": JSON.stringify({
                "host": data["host"],
                "path": file["full_name"]
              }),
            },
            {
              "name": "Delete",
              "type": "task",
              "ui_feature": "file_browser:remove",
              "parameters": JSON.stringify({
                "host": data["host"],
                "path": file["full_name"],
              }),
              "startIcon": "delete"
            },
            {
              "name": "Download",
              "type": "task",
              "disabled": !file["is_file"],
              "ui_feature": "file_browser:download",
              "parameters": JSON.stringify({
                "host": data["host"],
                "file": file["full_name"],
              }),
              "startIcon": "download"
            },
            {
              "name": "List",
              "type": "task",
              "ui_feature": "file_browser:list",
              "disabled": file["is_file"],
              "parameters": JSON.stringify({
                "host": data["host"],
                "path": file["full_name"],
              }),
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

    if (platform == "Windows") {
      let acls = file["permissions"]["acls"].map((acl) => ({
        "account": {
          "plaintext": acl["account"]
        },

        "rights": {
          "plaintext": acl["rights"]
        },

        "type": {
          "plaintext": acl["type"]
        },
      }));

      row["metadata"] = {
        "button": {
          "name": "view",
          "type": "table",
          "title": `Metadata for ${file["name"]}`,
          "startIcon": "list",
          "hoverText": `View ${file["is_file"] ? "file" : "directory"} metadata`,
          "value": {
            "headers": [
              { "plaintext": "account", "width": 400, "type": "string" },
              { "plaintext": "rights", "fillWidth": true, "type": "string" },
              { "plaintext": "type", "width": 400, "type": "string" },
            ],
            "rows": acls
          }
        }
      };

      row["owner"] = {
        "plaintext": file["owner"],
        "copyIcon": true
      }

    } else if (platform == "Linux") {
      let metadataInfo = {
        "Created on": created,
        "Last accessed": accessed,
        "Last modified": modified,
        "User ownership": `${file["permissions"]["user"]}(${file["permissions"]["uid"]})`,
        "Group ownership": `${file["permissions"]["user"]}(${file["permissions"]["uid"]})`,
        "Permissions": file["permissions"]["permissions"]
      };

      if (file["is_file"]) {
        metadataInfo["File is readable"] = file["permissions"]["permissions"][1] == 'r' ? "true" : "false";
        metadataInfo["File is writable"] = file["permissions"]["permissions"][2] == 'w' ? "true" : "false";
      } else {
        if (file["permissions"]["permissions"][1] == 'r' && file["permissions"]["permissions"][3] == 'x') {
          metadataInfo["Directory is listable"] = "true";
        } else {
          metadataInfo["Directory is listable"] = "false";
        }
      }

      row["metadata"] = {
        "button": {
          "name": "view",
          "type": "dictionary",
          "title": `Metadata for ${file["name"]}`,
          "leftColumnTitle": "Attribute",
          "rightColumnTitle": "Value",
          "value": metadataInfo,
          "startIcon": "list",
          "hoverText": `View ${file["is_file"] ? "file" : "directory"} metadata`
        }
      };

      row["user"] = {
        "plaintext": file["permissions"]["user"],
        "copyIcon": true
      };

      row["group"] = {
        "plaintext": file["permissions"]["group"],
        "copyIcon": true
      };

      row["permissions"] = {
        "plaintext": file["permissions"]["permissions"]
      };
    }

    rows.push(row);

  }

  return {
    "table": [
      {
        "headers": headers,
        "rows": rows
      }
    ]
  }
}
