/**
 * @file string.hpp
 * Vita::string declaration.
 *
 * Licensed under the terms of the MIT/X11 license.
 * Copyright (c) 2008 Vita Smid <me@ze.phyr.us>
 *
 * $Id: string.hpp 17 2008-08-11 17:46:13Z zephyr $
 */

#ifndef INC_VITA_STRING_HPP
#define INC_VITA_STRING_HPP

#include <sstream>
#include <string>
#include <string_view>
#include <vector>

namespace Vita {

/**
 * Slightly enhanced version of std::string.
 */
class string : public std::string {
 public:
    /**
     * Directly call <tt>std::string::string()</tt>.
     */
    explicit string() : std::string(){};

    /**
     * Directly call <tt>std::string::string(const char*)</tt>.
     */
    string(const char* cstr) : std::string(cstr){};
    string(const std::string_view& str) : std::string(str.data()){};

    /**
     * Directly call <tt>std::string::string(const char*, size_t)</tt>.
     */
    string(const char* cstr, size_t n) : std::string(cstr, n){};

    /**
     * Directly call <tt>std::string::string(const std::string&)</tt>.
     */
    string(const std::string& str) : std::string(str){};

    /**
     * Directly call <tt>std::string::string(const std::string&, size_t, size_t)</tt>.
     */
    string(const std::string& str, size_t pos, size_t n = npos) : std::string(str, pos, n){};

    /**
     * Directly call <tt>std::string::string(size_t, char)</tt>.
     */
    string(size_t n, char c) : std::string(n, c){};

    /**
     * Convert all characters to lower case.
     */
    string toLower() const;

    /**
     * Convert all characters to upper case.
     */
    string toUpper() const;

    /**
     * Make the first character uppercase.
     */
    string ucfirst() const;

    /**
     * Make the first character lowercase.
     */
    string lcfirst() const;

    /**
     * Convert the operand to string and append it.
     *
     * This overrides the behavior of std::string.
     *
     * @param operand The number to be appended.
     * @return The string with @a operand appended.
     */
    string operator+(int operand) const;

    /**
     * Convert the operand to string and append it.
     *
     * This overrides the behavior of std::string.
     *
     * @param operand The number to be appended.
     * @return The string with @a operand appended.
     */
    string operator+(long int operand) const;

    /**
     * Convert the operand to string and append it.
     *
     * This overrides the behavior of std::string.
     *
     * @param operand The number to be appended.
     * @return The string with @a operand appended.
     */
    string operator+(double operand) const;

    /**
     * Convert the operand to string and append it.
     *
     * This overrides the behavior of std::string.
     *
     * @param operand The number to be appended.
     * @return The string with @a operand appended.
     */
    string operator+(float operand) const;

    /**
     * Replace all occurences of a certain substring in the string.
     *
     * @param search The substring that will be replaced.
     * @param replace The replacement.
     * @param limit How many replacements should be done. Set to Vita::string::npos to disable the limit.
     * @return String with the replacement(s) in place.
     */
    string replace(const std::string_view& search, const std::string_view& replace, size_t limit = npos) const;

    /**
     * Split the string by another string.
     *
     * This method is similar to the <tt>explode</tt> function known from PHP.
     *
     * @param delimiter The boundary string.
     * @return A vector of strings, each of which is a substring of the original.
     */
    std::vector<string> explode(const std::string_view& delimiter) const;

    /**
     * Trim unwanted characters from the beginning and the end of the string.
     *
     * @param what The characters to trim. Defaults to whitespace (ASCII #9, #10, #13, #32).
     * @return The trimmed string.
     */
    string trim(const std::string_view& what = "\x9\xa\xd\x20") const;

    /**
     * Convert a generic data type to string.
     *
     * The conversion is done via std::ostringstream.
     *
     * @param source The value to convert.
     * @return string
     */
    template <class T>
    static string toStr(const T& source) {
        std::ostringstream stream;
        stream << source;
        return string(stream.str());
    }

    /**
     * Convert the string to a generic data type.
     *
     * The conversion is done via std::istringstream.
     *
     * @return The converted string.
     */
    template <class T>
    T convert() const {
        std::istringstream stream(*this);
        T result;
        stream >> result;
        return result;
    }
};

}  // namespace Vita

#endif  // INC_VITA_STRING_HPP
