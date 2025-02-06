(concatenate-manifests
 (list
  (package->development-manifest
   (specification->package "emacs-next-minimal"))
  (specifications->manifest
   (list  "gnutls"
	  "mesa"
	  "cairo"
	  "wayland"
	  "wayland-protocols"
	  "libxkbcommon"
	  "freetype"
	  "harfbuzz"
	  "fontconfig"
	  "clang"
	  "libgccjit"
	  "gcc-toolchain"
	  ;; lsp
	 "bear"
	 "gdb"
	 "valgrind"
	 "strace"
	 "glibc:debug")
   )))
