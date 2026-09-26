# The build and run toolchain (tools/regress-docker.sh builds it as eech-build).
# Debian bookworm's GCC 12: GCC 11 lacks -ftrivial-auto-var-init, and GCC 14
# makes the original sources' incompatible-pointer warnings errors.
#
# Windows: MinGW-w64 (the same GCC 12, POSIX threads) cross-compiles
# eech_dc.dll and eech-world.exe for x86_64-pc-windows-gnu (tools/build-windows.sh),
# against Lua 5.1.5 built as lua.dll: the name DCS World's own Lua 5.1 has, so
# the module links the same way in both hosts.
FROM rust:bookworm
RUN apt-get update \
	&& apt-get install -y --no-install-recommends liblua5.1-0-dev zip gdb python3 gcc-mingw-w64-x86-64 \
	&& rm -rf /var/lib/apt/lists/* \
	&& update-alternatives --set x86_64-w64-mingw32-gcc /usr/bin/x86_64-w64-mingw32-gcc-posix \
	&& rustup target add x86_64-pc-windows-gnu
RUN curl -fsSL -o /tmp/lua.tar.gz https://www.lua.org/ftp/lua-5.1.5.tar.gz \
	&& echo "2640fc56a795f29d28ef15e13c34a47e223960b0240e8cb0a82d9b0738695333  /tmp/lua.tar.gz" | sha256sum -c - \
	&& mkdir -p /opt/lua-mingw/include && cd /tmp && tar xzf lua.tar.gz && cd lua-5.1.5/src \
	&& x86_64-w64-mingw32-gcc -O2 -DLUA_BUILD_AS_DLL -c $(ls *.c | grep -v -E '^(lua|luac|print)\.c$') \
	&& x86_64-w64-mingw32-gcc -shared -o /opt/lua-mingw/lua.dll *.o -Wl,--out-implib,/opt/lua-mingw/liblua.dll.a \
	&& cp lua.h luaconf.h lualib.h lauxlib.h /opt/lua-mingw/include/ \
	&& rm -rf /tmp/lua*
