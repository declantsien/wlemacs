(concatenate-manifests
 (list
  (package->development-manifest
   (specification->package "emacs-next-minimal"))
  (specifications->manifest
   (list  "gnutls"
	  "mesa"
	  "wayland"
	  "wayland-protocols"
	  "libxkbcommon"
	  "freetype"
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
