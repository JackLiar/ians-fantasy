add_rules("mode.debug", "mode.release")

if is_os("linux") then 
    add_requires("alsa-lib")
    add_requires("libudev")
    add_requires("wayland 1.23.1")
else 
end


target("ians-fantasy")
    set_kind("phony")
    if is_os("linux") then 
        add_packages("alsa-lib")
        add_packages("libudev")
        add_packages("wayland")
    else 
    end
    after_build(function(target)
        local pkg_config_dirs = {}
        local include_dirs = {}
        local link_dirs = {}
        for _, pkg in pairs(target:pkgs()) do
            -- table.append(pkg_config_dirs, path.join(pkg:installdir(), "lib", "pkgconfig"))
            table.append(include_dirs, path.join(pkg:installdir(), "include"))
            table.append(link_dirs, path.join(pkg:installdir(), "lib"))
            if os.exists(path.join(pkg:installdir(), "lib64")) then
                table.append(link_dirs, path.join(pkg:installdir(), "lib64"))
            end
            for _, dir in ipairs(pkg._INFO.linkdirs) do
                table.append(pkg_config_dirs, path.join(dir, "pkgconfig"))
            end

        end
        if os.is_host("windows") then 
            function gen_powershell_env()
                local ps_env = io.open("env.ps1", "w")
                ps_env:write("$env:PKG_CONFIG_PATH=\"" .. table.concat(pkg_config_dirs, ";") .. ";$env:PKG_CONFIG_PATH\"\n")
                ps_env:write("$env:C_INCLUDE_PATH=\"" .. table.concat(include_dirs, ";") .. ";$env:C_INCLUDE_PATH\"\n")
                ps_env:write("$env:CPLUS_INCLUDE_PATH=\"" .. table.concat(include_dirs, ";") .. ";$env:CPLUS_INCLUDE_PATH\"\n")
                ps_env:write("$env:LIBRARY_PATH=\"" .. table.concat(link_dirs, ";") .. ";$env:LIBRARY_PATH\"\n")
                ps_env:write("$env:PATH=\"" .. table.concat(link_dirs, ";") .. ";$env:PATH\"\n")
                ps_env:close()
            end
            gen_powershell_env()
        else
            function gen_bash_env()
                local bash_env = io.open("env.sh", "w")
                bash_env:write("#!/bin/bash\n")
                bash_env:write("export PKG_CONFIG_PATH=\"" .. table.concat(pkg_config_dirs, ":") .. ":$PKG_CONFIG_PATH\"\n")
                bash_env:write("export C_INCLUDE_PATH=\"" .. table.concat(include_dirs, ":") .. ":$C_INCLUDE_PATH\"\n")
                bash_env:write("export CPLUS_INCLUDE_PATH=\"" .. table.concat(include_dirs, ":") .. ":$CPLUS_INCLUDE_PATH\"\n")
                bash_env:write("export LIBRARY_PATH=\"" .. table.concat(link_dirs, ":") .. ":$LIBRARY_PATH\"\n")
                bash_env:write("export LD_LIBRARY_PATH=\"" .. table.concat(link_dirs, ":") .. ":$LD_LIBRARY_PATH\"\n")
                bash_env:close()
            end
            gen_bash_env()
            function gen_fish_env()
                local fish_env = io.open("env.fish", "w")
                fish_env:write("set -x PKG_CONFIG_PATH " .. table.concat(pkg_config_dirs, " ") .. " $PKG_CONFIG_PATH\n")
                fish_env:write("set -x C_INCLUDE_PATH " .. table.concat(include_dirs, " ") .. " $C_INCLUDE_PATH\n")
                fish_env:write("set -x CPLUS_INCLUDE_PATH " .. table.concat(include_dirs, " ") .. " $CPLUS_INCLUDE_PATH\n")
                fish_env:write("set -x LIBRARY_PATH " .. table.concat(link_dirs, " ") .. " $LIBRARY_PATH\n")
                fish_env:write("set -x LD_LIBRARY_PATH " .. table.concat(link_dirs, " ") .. " $LD_LIBRARY_PATH\n")
                fish_env:close()
            end
            gen_fish_env()
        end
    end)