/*
 * The MIT License (MIT)
 * Copyright (c) 2018 Danijel Durakovic
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
 * of the Software, and to permit persons to whom the Software is furnished to do
 * so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 *
 */

///////////////////////////////////////////////////////////////////////////////
//
//  /mINI/ v0.9.11
//  An INI file reader and writer for the modern age.
//
///////////////////////////////////////////////////////////////////////////////
//
//  A tiny utility library for manipulating INI files with a straightforward
//  API and a minimal footprint. It conforms to the (somewhat) standard INI
//  format - sections and keys are case insensitive and all leading and
//  trailing whitespace is ignored. Comments are lines that begin with a
//  semicolon. Trailing comments are allowed on section lines.
//
//  Files are read on demand, upon which data is kept in memory and the file
//  is closed. This utility supports lazy writing, which only writes changes
//  and updates to a file and preserves custom formatting and comments. A lazy
//  write invoked by a write() call will read the output file, find what
//  changes have been made and update the file accordingly. If you only need to
//  generate files, use generate() instead. Section and key order is preserved
//  on read, write and insert.
//
///////////////////////////////////////////////////////////////////////////////
//
//  Long live the INI file!!!
//
///////////////////////////////////////////////////////////////////////////////

#ifndef MINI_INI_HPP_
#define MINI_INI_HPP_

#include <algorithm>
#include <cctype>
#include <fstream>
#include <memory>
#include <sstream>
#include <string>
#include <sys/stat.h>
#include <unordered_map>
#include <utility>
#include <vector>

namespace mINI {
namespace INIStringUtil {
    static constexpr auto whitespaceDelimiters = " \t\n\r\f\v";
    inline void trim(std::string& str) noexcept {
        str.erase(str.find_last_not_of(whitespaceDelimiters) + 1);
        str.erase(0, str.find_first_not_of(whitespaceDelimiters));
    }
#ifndef MINI_CASE_SENSITIVE
    inline void toLower(std::string& str) noexcept {
        std::transform(str.begin(), str.end(), str.begin(), [](const char c) {
            return static_cast<char>(std::tolower(c));
        });
    }
#endif
    inline void replace(std::string& str, const std::string_view& a, const std::string_view& b) noexcept {
        if (!a.empty()) {
            std::size_t pos = 0;
            while ((pos = str.find(a, pos)) != std::string::npos) {
                str.replace(pos, a.size(), b);
                pos += b.size();
            }
        }
    }
}  // namespace INIStringUtil

template <typename T>
class INIMap {
 private:
    using T_DataIndexMap  = std::unordered_map<std::string, std::size_t>;
    using T_DataItem      = std::pair<std::string, T>;
    using T_DataContainer = std::vector<T_DataItem>;
    using T_MultiArgs     = typename std::vector<std::pair<std::string, T>>;

    T_DataIndexMap dataIndexMap{};
    T_DataContainer data{};

    constexpr std::size_t setEmpty(std::string& key) noexcept {
        const auto& index = data.size();
        dataIndexMap[key] = index;
        data.emplace_back(key, T());
        return index;
    }

 public:
    using const_iterator = typename T_DataContainer::const_iterator;

    INIMap() = default;

    INIMap(const INIMap& other) noexcept {
        const auto& data_size = other.data.size();
        for (std::size_t i = 0; i < data_size; ++i) {
            const auto& key = other.data[i].first;
            const auto& obj = other.data[i].second;
            data.emplace_back(key, obj);
        }
        dataIndexMap = T_DataIndexMap(other.dataIndexMap);
    }

    inline T& operator[](std::string key) noexcept {
        INIStringUtil::trim(key);
#ifndef MINI_CASE_SENSITIVE
        INIStringUtil::toLower(key);
#endif
        const auto& it    = dataIndexMap.find(key);
        const auto& hasIt = (it != dataIndexMap.end());
        const auto& index = (hasIt) ? it->second : setEmpty(key);
        return data[index].second;
    }
    constexpr T get(std::string& key) noexcept {
        INIStringUtil::trim(key);
#ifndef MINI_CASE_SENSITIVE
        INIStringUtil::toLower(key);
#endif
        const auto& it = dataIndexMap.find(key);
        if (it == dataIndexMap.end()) {
            return T();
        }
        return T(data[it->second].second);
    }
    constexpr bool has(std::string& key) const noexcept {
        INIStringUtil::trim(key);
#ifndef MINI_CASE_SENSITIVE
        INIStringUtil::toLower(key);
#endif
        return (dataIndexMap.count(key) == 1);
    }
    constexpr void set(std::string& key, T obj) {
        INIStringUtil::trim(key);
#ifndef MINI_CASE_SENSITIVE
        INIStringUtil::toLower(key);
#endif
        const auto& it = dataIndexMap.find(key);
        if (it != dataIndexMap.end()) {
            data[it->second].second = obj;
            return;
        }
        dataIndexMap[key] = data.size();
        data.emplace_back(key, obj);
    }
    constexpr void set(const T_MultiArgs& multiArgs) noexcept {
        for (const auto& it : multiArgs) {
            const auto& key = it.first;
            const auto& obj = it.second;
            set(key, obj);
        }
    }
    inline bool remove(std::string& key) noexcept {
        INIStringUtil::trim(key);
#ifndef MINI_CASE_SENSITIVE
        INIStringUtil::toLower(key);
#endif
        const auto& it = dataIndexMap.find(key);
        if (it != dataIndexMap.end()) {
            const auto& index = it->second;
            data.erase(data.begin() + index);
            dataIndexMap.erase(it);
            for (auto& it2 : dataIndexMap) {
                auto& vi = it2.second;
                if (vi > index) {
                    vi--;
                }
            }
            return true;
        }
        return false;
    }
    constexpr void clear() noexcept {
        data.clear();
        dataIndexMap.clear();
    }
    constexpr std::size_t size() const noexcept {
        return data.size();
    }
    constexpr const_iterator begin() const noexcept { return data.begin(); }
    constexpr const_iterator end() const noexcept { return data.end(); }
};

using INIStructure = INIMap<INIMap<std::string>>;

namespace INIParser {
    using T_ParseValues = std::pair<std::string, std::string>;

    enum class PDataType : int8_t {
        PDATA_NONE,
        PDATA_COMMENT,
        PDATA_SECTION,
        PDATA_KEYVALUE,
        PDATA_UNKNOWN
    };

    PDataType parseLine(std::string line, T_ParseValues& parseData) noexcept {
        parseData.first.clear();
        parseData.second.clear();
        INIStringUtil::trim(line);
        if (line.empty() || line.starts_with('#')) {
            return PDataType::PDATA_NONE;
        }
        const char firstCharacter = line[0];
        if (firstCharacter == ';') {
            return PDataType::PDATA_COMMENT;
        }
        if (firstCharacter == '[') {
            const auto& commentAt = line.find_first_of(';');
            if (commentAt != std::string::npos) {
                line = line.substr(0, commentAt);
            }
            const auto& closingBracketAt = line.find_last_of(']');
            if (closingBracketAt != std::string::npos) {
                auto section = line.substr(1, closingBracketAt - 1);
                INIStringUtil::trim(section);
                parseData.first = section;
                return PDataType::PDATA_SECTION;
            }
        }
        auto lineNorm = line;
        INIStringUtil::replace(lineNorm, "\\=", "  ");
        const auto& equalsAt = lineNorm.find_first_of('=');
        if (equalsAt != std::string::npos) {
            auto key = line.substr(0, equalsAt);
            INIStringUtil::trim(key);
            INIStringUtil::replace(key, "\\=", "=");
            auto value = line.substr(equalsAt + 1);
            INIStringUtil::trim(value);
            parseData.first  = key;
            parseData.second = value;
            return PDataType::PDATA_KEYVALUE;
        }
        return PDataType::PDATA_UNKNOWN;
    }
}  // namespace INIParser

class INIReader {
 public:
    using T_LineData    = std::vector<std::string>;
    using T_LineDataPtr = std::shared_ptr<T_LineData>;

 private:
    std::ifstream fileReadStream{};
    T_LineDataPtr lineData{};

    T_LineData readFile() noexcept {
        std::string fileContents{};
        fileReadStream.seekg(0, std::ios::end);
        fileContents.resize(static_cast<std::size_t>(fileReadStream.tellg()));
        fileReadStream.seekg(0, std::ios::beg);
        const auto& fileSize = fileContents.size();
        T_LineData output{};
        if (fileSize == 0) {
            return output;
        }

        fileReadStream.read(&fileContents[0], static_cast<ssize_t>(fileSize));
        fileReadStream.close();

        std::string buffer{};
        buffer.reserve(256);
        for (std::size_t i = 0; i < fileSize; ++i) {
            const auto& c = fileContents[i];
            if (c == '\n') {
                output.emplace_back(buffer);
                buffer.clear();
                continue;
            }
            if (c != '\0' && c != '\r') {
                buffer += c;
            }
        }
        output.emplace_back(buffer);
        return output;
    }

 public:
    explicit INIReader(const std::string_view& filename, bool keepLineData = false) noexcept {
        fileReadStream.open(filename.data(), std::ios::in | std::ios::binary);
        if (keepLineData) {
            lineData = std::make_shared<T_LineData>();
        }
    }
    ~INIReader() = default;

    bool operator>>(INIStructure& data) noexcept {
        if (!fileReadStream.is_open()) {
            return false;
        }
        auto fileLines = readFile();
        std::string section{};
        bool inSection = false;
        int repeated{};
        INIParser::T_ParseValues parseData;
        for (const auto& line : fileLines) {
            const auto& parseResult = INIParser::parseLine(line, parseData);
            if (parseResult == INIParser::PDataType::PDATA_SECTION) {
                inSection = true;
                data[section = parseData.first];
            } else if (inSection && parseResult == INIParser::PDataType::PDATA_KEYVALUE) {
                const auto& key    = parseData.first;
                const auto& value  = parseData.second;
                data[section][key] = value;
            } else if (parseResult == INIParser::PDataType::PDATA_KEYVALUE) {
                const auto& key                     = parseData.first;
                const auto& value                   = parseData.second;
                data[std::to_string(repeated)][key] = value;
                ++repeated;
            }
            if (lineData && parseResult != INIParser::PDataType::PDATA_UNKNOWN) {
                if (parseResult == INIParser::PDataType::PDATA_KEYVALUE && !inSection) {
                    continue;
                }
                lineData->emplace_back(line);
            }
        }
        return true;
    }
    T_LineDataPtr getLines() const noexcept {
        return lineData;
    }
};

class INIFile {
 private:
    std::string_view m_filename{};

 public:
    explicit INIFile(const std::string_view& filename)
      : m_filename(filename) { }

    ~INIFile() = default;

    bool read(INIStructure& data) const noexcept {
        if (data.size()) {
            data.clear();
        }
        INIReader reader(m_filename);
        return reader >> data;
    }
};
}  // namespace mINI

#endif  // MINI_INI_HPP_
