/* lua-debug-layout: the size of the lua_Debug that the Lua 5.1 in a lua.dll writes.
 *
 * A native function fills a buffer with a canary, lets lua_getstack and lua_getinfo ("Sln")
 * write the record for itself, and reports the end of the last byte that changed. Two canaries,
 * so a written byte equal to one of them cannot end the record early. Compare the result with
 * the record mlua-sys allocates (docs/m5-dcs-lua-abi.md). Build: build.sh; run: lua-debug-layout.exe
 * beside the lua.dll to measure.
 */
#include <stdio.h>
#include <string.h>

#include "lauxlib.h"
#include "lua.h"
#include "lualib.h"

/* far larger than any Lua 5.1 lua_Debug; static, so a large record cannot overrun the C stack */
#define RECORD_LIMIT 65536

static unsigned char record[RECORD_LIMIT];

/* the end of the bytes lua_getstack and lua_getinfo changed; 0 if no function is running */
static size_t bytes_written(lua_State *L, unsigned char canary)
{
	size_t end = sizeof record;

	memset(record, canary, sizeof record);
	if (!lua_getstack(L, 0, (lua_Debug *)record))
		return 0;
	lua_getinfo(L, "Sln", (lua_Debug *)record);
	while (end > 0 && record[end - 1] == canary)
		end--;
	return end;
}

static int record_size(lua_State *L)
{
	size_t a = bytes_written(L, 0xA5), b = bytes_written(L, 0x5A);
	size_t size = a > b ? a : b;

	if (a == 0 || b == 0)
		return luaL_error(L, "lua_getstack found no running function");
	if (size == sizeof record)
		return luaL_error(L, "the record reaches the %d-byte limit: its size is unknown", RECORD_LIMIT);
	lua_pushinteger(L, (lua_Integer)size);
	return 1;
}

int main(void)
{
	lua_State *L = luaL_newstate();
	int status;

	if (!L) {
		fputs("lua-debug-layout: luaL_newstate failed\n", stderr);
		return 2;
	}
	luaL_openlibs(L);
	lua_register(L, "record_size", record_size);
	status = luaL_dostring(L, "print(_VERSION .. ' lua_Debug bytes written: ' .. record_size())");
	if (status)
		fprintf(stderr, "lua-debug-layout: %s\n", lua_tostring(L, -1));
	lua_close(L);
	return status ? 1 : 0;
}
