function(task, responses) {
  if (task.status.includes("error")) {
    const combined = responses.reduce((prev, cur) => {
      return prev + cur;
    }, "");
    return { "plaintext": combined };
  }

  let task_params = {};
  try {
    task_params = JSON.parse(task.original_params);
  } catch (error) {
    console.log(error);
    const combined = responses.reduce((prev, cur) => {
      return prev + cur;
    }, "");

    return { "plaintext": combined };
  }

  if (responses.length > 0) {
    if (task_params.hasOwnProperty("list")) {

      const DAYS_OF_WEEK = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
      const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
      var archiveFormats = [".a", ".ar", ".cpio", ".shar", ".LBR", ".lbr", ".mar", ".sbx", ".tar", ".bz2", ".F", ".gz", ".lz", ".lz4", ".lzma", ".lzo", ".rz", ".sfark", ".sz", ".?Q?", ".?Z?", ".xz", ".z", ".Z", ".zst", ".??", ".7z", ".s7z", ".ace", ".afa", ".alz", ".apk", ".arc", ".arc", ".arj", ".b1", ".b6z", ".ba", ".bh", ".cab", ".car", ".cfs", ".cpt", ".dar", ".dd", ".dgc", ".ear", ".gca", ".ha", ".hki", ".ice", ".jar", ".kgb", ".lzh", ".lzx", ".pak", ".pak", ".parti", ".paq6", ".pea", ".pim", ".pit", ".qda", ".rar", ".rk", ".sda", ".sea", ".sen", ".sfx", ".shk", ".sit", ".sitx", ".sqx", ".tar", ".tbz2", ".uc", ".uca", ".uha", ".war", ".wim", ".xar", ".xp3", ".yz1", ".zip", ".zoo", ".zpaq", ".zz", ".ecc", ".ecsbx", ".par", ".par2", ".rev"];
      var diskImages = [".dmg", ".iso", ".vmdk"];
      var wordDocs = [".doc", ".docx", ".dotm", ".dot", ".wbk", ".docm", ".dotx", ".docb"];
      var excelDocs = [".csv", ".xls", ".xlsx", ".xlsm", ".xltx", ".xltm", ".xlmx", ".xlmt"];
      var powerPoint = [".ppt", ".pptx", ".potx", ".ppsx", ".thmx", ".pot", ".pps"];
      var pdfExt = [".pdf"];
      var dbExt = [".db", ".sql", ".psql"];
      var keyFiles = [".pem", ".ppk", ".cer", ".pvk", ".pfx"];
      var codeFiles = [".config", ".ps1", ".psm1", ".psd1", ".vbs", ".js", ".py", ".pl", ".rb", ".go", ".xml", ".html", ".css", ".sh", ".bash", ".yaml", ".yml", ".c", ".cpp", ".h", ".hpp", ".cs", ".sln", ".csproj"];
      var imageFiles = [".2000", ".ani", ".anim", ".apng", ".art", ".avif", ".bmp", ".bpg", ".bsave", ".cal", ".cin", ".cpc", ".cpt", ".cur", ".dds", ".dpx", ".ecw", ".ep", ".exr", ".fits", ".flic", ".flif", ".fpx", ".gif", ".hdr", ".hdri", ".hevc", ".icer", ".icns", ".ico", ".ics", ".ilbm", ".it", ".jbig", ".jbig2", ".jng", ".jpeg", ".jpeg", ".jpeg", ".jpeg", ".jpeg", ".jpeg", ".jpeg", ".jpeg", ".kra", ".logluv", ".ls", ".miff", ".mng", ".nrrd", ".pam", ".pbm", ".pcx", ".pgf", ".pgm", ".pictor", ".png", ".pnm", ".ppm", ".psb", ".psd", ".psp", ".qtvr", ".ras", ".rgbe", ".sgi", ".tga", ".tiff", ".tiff", ".tiff", ".tiff", ".ufo", ".ufp", ".wbmp", ".webp", ".xbm", ".xcf", ".xl", ".xpm", ".xr", ".xs", ".xt", ".xwd"];
      let file = {};
      let data = {};
      let rows = [];
      let tableHeader = "";
      let headers = [];

      for (var i = 0; i < responses.length; i++) {
        try {
          data = JSON.parse(responses[i]);
        } catch (error) {
          console.log(error);
          const combined = responses.reduce((prev, cur) => {
            return prev + cur;
          }, "");
          return { 'plaintext': combined };
        }

        headers = [
          { "plaintext": "actions", "type": "button", "cellStyle": {}, "width": 120, "disableSort": true },
          { "plaintext": "Task", "type": "button", "cellStyle": {}, "width": 100, "disableSort": true },
          { "plaintext": "name", "type": "string", "fillWidth": true, "cellStyle": {} },
          { "plaintext": "size", "type": "size", "width": 125, "cellStyle": {} },
          { "plaintext": "last modified", "type": "string", "width": 250, "cellStyle": {} },
          { "plaintext": "last accessed", "type": "string", "width": 250, "cellStyle": {} },
          { "plaintext": "uid", "type": "string", "width": 100, "cellStyle": {} },
          { "plaintext": "gid", "type": "string", "width": 100, "cellStyle": {} },
          { "plaintext": "permissions", "type": "string", "fillWidth": true, "cellStyle": {} },
        ];

        let ls_path = "";
        if (data["parent_path"] == "") {
          ls_path = data["name"];
        } else if (data["parent_path"] == "/") {
          ls_path = "/" + data["name"];
        } else {
          ls_path = data["parent_path"] + "/" + data["name"];
        }

        tableHeader = "Listing for " + task_params["credentials"]["account"] + '@' + task_params["host"] + ':' + ls_path;

        for (let j = 0; j < data["files"].length; j++) {
          let finfo = data["files"][j];
          let buttonSettings = {};
          let startIcon = "";
          let startIconHoverText = "";
          let startIconColor = "";
          if (finfo["is_file"]) {
            var fileExt = "." + finfo['name'].split(".").slice(-1)[0].toLowerCase();
            if (archiveFormats.includes(fileExt)) {
              startIcon = 'archive';
              startIconHoverText = "Archive File";
              startIconColor = "goldenrod";
            } else if (diskImages.includes(fileExt)) {
              startIcon = 'diskimage';
              startIconHoverText = "Disk Image";
              startIconColor = "goldenrod";
            } else if (wordDocs.includes(fileExt)) {
              startIcon = 'word';
              startIconHoverText = "Microsoft Word Document";
              startIconColor = "cornflowerblue";
            } else if (excelDocs.includes(fileExt)) {
              startIcon = 'excel';
              startIconHoverText = "Microsoft Excel Document";
              startIconColor = "darkseagreen";
            } else if (powerPoint.includes(fileExt)) {
              startIcon = 'powerpoint';
              startIconHoverText = "Microsoft PowerPoint Document";
              startIconColor = "indianred";
            } else if (pdfExt.includes(fileExt)) {
              startIcon = 'pdf';
              startIconHoverText = "Adobe Acrobat PDF";
              startIconColor = "orangered";
            } else if (dbExt.includes(fileExt)) {
              startIcon = 'database';
              startIconHoverText = "Database File Format";
            } else if (keyFiles.includes(fileExt)) {
              startIcon = 'key';
              startIconHoverText = "Key Credential Material";
            } else if (codeFiles.includes(fileExt)) {
              startIcon = 'code';
              startIconHoverText = "Source Code";
              startIconColor = "rgb(25,142,117)";
            } else if (imageFiles.includes(fileExt)) {
              startIcon = "image";
              startIconHoverText = "Image File";
            }

            let cat_parameters = {
              credentials: task_params["credentials"],
              agent: task_params["agent"],
              host: task_params["host"],
              port: task_params["port"],
              cat: finfo["full_name"],
            };

            buttonSettings = {
              "button": {
                "name": "cat",
                "type": "task",
                "ui_feature": "ssh",
                "parameters": cat_parameters,
              },
              "cellStyle": {},
            }

          } else {
            let ls_parameters = {
              credentials: task_params["credentials"],
              agent: task_params["agent"],
              host: task_params["host"],
              port: task_params["port"],
              list: finfo["full_name"],
            };

            startIcon = "openFolder";
            startIconHoverText = "Directory";
            startIconColor = "rgb(241,226,0)";
            buttonSettings = {
              "button": {
                "name": "ls",
                "type": "task",
                "parameters": ls_parameters,
                "ui_feature": "ssh",
                "startIcon": "list",
              },
              "cellStyle": {},
            }
          }

          let date = new Date(parseInt(data["files"][j]["modify_time"]) * 1000);
          let dow = DAYS_OF_WEEK[date.getDay()];
          let month = MONTHS[date.getMonth()];

          let last_modified_date = dow + ', ';
          last_modified_date += String(date.getDate() + ' ');
          last_modified_date += month + ' ';
          last_modified_date += String(date.getFullYear() + ' ');
          last_modified_date += String(date.getHours() + ':');
          last_modified_date += String(date.getMinutes()).padStart(2, '0');

          date = new Date(parseInt(data["files"][j]["access_time"]) * 1000);
          dow = DAYS_OF_WEEK[date.getDay()];
          month = MONTHS[date.getMonth()];

          let access_date = dow + ', ';
          access_date += String(date.getDate() + ' ');
          access_date += month + ' ';
          access_date += String(date.getFullYear() + ' ');
          access_date += String(date.getHours() + ':');
          access_date += String(date.getMinutes()).padStart(2, '0');

          let download_parameters = {
            credentials: task_params["credentials"],
            agent: task_params["agent"],
            host: task_params["host"],
            port: task_params["port"],
            download: finfo["full_name"],
          };

          let rm_parameters = {
            credentials: task_params["credentials"],
            agent: task_params["agent"],
            host: task_params["host"],
            port: task_params["port"],
            rm: finfo["full_name"],
          };

          let row = {
            "rowStyle": {},
            "actions": {
              "button": {
                "startIcon": "list",
                "name": "Actions",
                "type": "menu",
                "value": [
                  {
                    "name": "Download",
                    "type": "task",
                    "disabled": !finfo["is_file"],
                    "ui_feature": "ssh",
                    "parameters": download_parameters,
                    "startIcon": "download",
                  },
                  {
                    "name": "Delete",
                    "type": "task",
                    "ui_feature": "ssh",
                    "parameters": rm_parameters,
                    "startIcon": "delete",
                  },
                ],
              },
            },
            "Task": buttonSettings,
            "name": {
              "plaintext": data["files"][j]["name"],
              "cellStyle": {},
              "startIcon": startIcon,
              "startIconHoverText": startIconHoverText,
              "startIconColor": startIconColor
            },
            "size": { "plaintext": data["files"][j]["size"], "cellStyle": {} },
            "uid": { "plaintext": data["files"][j]["permissions"]["uid"].toString(), "cellStyle": {} },
            "gid": { "plaintext": data["files"][j]["permissions"]["gid"].toString(), "cellStyle": {} },
            "permissions": { "plaintext": data["files"][j]["permissions"]["permissions"], "cellStyle": {} },
            "last modified": { "plaintext": last_modified_date, "cellStyle": {} },
            "last accessed": { "plaintext": access_date, "cellStyle": {} },
          };

          rows.push(row);
        }
      }

      return {
        "table": [{
          "headers": headers,
          "rows": rows,
          "title": tableHeader,
        }]
      };

    } else if (task_params.hasOwnProperty("download")) {
      try {
        return {
          "download": [{
            "agent_file_id": responses[0],
            "variant": "contained",
            "name": "Download " + task["display_params"]
          }]
        };
      } catch (error) {
        const combined = responses.reduce((prev, cur) => {
          return prev + cur;
        }, "");
        return { 'plaintext': combined };
      }
    } else {
      return { "plaintext": responses[0] };
    }
  } else {
    return { "plaintext": "Waiting for respone..." }
  }
}
