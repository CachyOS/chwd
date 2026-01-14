#!/usr/bin/env lua
--[[
Copyright (C) 2025-2026 Vasiliy Stelmachenok for CachyOS

This program is free software; you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation; either version 2 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License along
with this program; if not, write to the Free Software Foundation, Inc.,
51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
--]]

local GENERATION_PATTERN = "^#define INTEL_([^_]+)_?.*_IDS.*"
local DEVICE_ID_PATTERN = "^%s*MACRO__%(0x([0-9A-Za-z]+),.*"

local function printf(msg, ...)
    print(msg:format(...))
end

local function die(msg, ...)
    printf(msg, ...)
    os.exit(1)
end

local function parse(file)
    local line = file:read("*l")
    local gen, id
    local result = {}

    while line do
        if line == "" then
            gen = nil
        else
            if gen == nil then
                gen = line:match(GENERATION_PATTERN)

                if gen and not result[gen] then
                    result[gen] = {}
                end
            else
                id = line:match(DEVICE_ID_PATTERN)

                if id then
                    result[gen][#result[gen] + 1] = string.lower(id)
                end
            end
        end

        line = file:read("*l")
    end

    return result
end

local function help()
    print([[
Dump PCI IDs of Intel GPUs from pciids.h

Usage: intel-pci-ids-dumper [OPTIONS] [GEN...]

Arguments:
  [GEN] - name of GPU generation

Options:
  -i, --input <PATH>  Input file with PCI IDs
  -o, --output <PATH> Output file for PCI IDs of specified generations
  -h, --help          Show this message
]]
    )
    os.exit(0)
end

local function main()
    local input, output
    local gens = {}

    if #arg < 1 then
        help()
    end

    local i = 1
    while i < #arg + 1 do
        if arg[i] == "--input" or arg[i] == "-i" then
            input = arg[i + 1]
            i = i + 1
        elseif arg[i] == "--help" or arg[i] == "-h" then
            help()
        elseif arg[i] == "--output" or arg[i] == "-o" then
            output = arg[i + 1]
            if not output then
                die("Output file must be specified")
            end
            i = i + 1
        else
            gens[#gens + 1] = arg[i]
        end
        i = i + 1
    end

    if not input then
        die("Input file must be specified")
    end

    local file, errmsg = io.open(input, "r")

    if not file then
        die("Failed to open %s file: %s", input, errmsg)
    end

    local result = parse(file)
    file:close()

    if not next(result) then
        die("Specified file is not valid and does not contain PCI IDs")
    end

    local dump = {}

    if #gens > 0 then
        for _, gen in ipairs(gens) do
            if result[gen] then
                printf("Found %s ids for %s generation", #result[gen], gen)
                for _, id in pairs(result[gen]) do
                    dump[#dump + 1] = id
                end
            else
                printf("No PCI IDs were found for %s generation", gen)
            end
        end
    else
        for gen, ids in pairs(result) do
            printf("Found %s ids for %s generation", #result[gen], gen)
            for _, id in pairs(ids) do
                dump[#dump + 1] = id
            end
        end
    end

    table.sort(dump)

    if output then
        file, errmsg = io.open(output, "w")

        if not file then
            die("Failed to open %s file: %s", output, errmsg)
        end

        printf("Saving %d ids to %s file", #dump, output)
        file:write(table.concat(dump, " "))
        file:close()
    end
end

main()
