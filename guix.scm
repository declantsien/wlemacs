;; The ‘guix.scm’ file for Guile, for use by ‘guix shell’.

(use-modules (guix)
	     ;; (guix utils)
             (guix git-download)  ;for ‘git-predicate’
             ((guix licenses) #:prefix license:)
	     (gnu packages)
	     (gnu packages xdisorg)
	     (gnu packages freedesktop)
	     (gnu packages gl)
	     (gnu packages image)
	     (gnu packages llvm)
	     (gnu packages commencement)	     
             (gnu packages autotools)
	     (gnu packages bash)
	     (gnu packages base)
	     (gnu packages cmake)
	     (gnu packages gawk)
	     (gnu packages pkg-config)
	     (gnu packages gtk)
	     (gnu packages glib)
	     (gnu packages tls)
	     (gnu packages version-control)
	     (gnu packages compression)
	     (gnu packages ncurses)
	     (guix build-system gnu)
	     (rustup build toolchain)
	     (guix build-system glib-or-gtk)
	     (gnu packages texinfo)
             (gnu packages emacs))

(define libxkbcommon-1.7
  (package
  (inherit libxkbcommon)
  (name "libxkbcommon")
  (version "1.7.0")
  ;; (source (origin
  ;;           (method url-fetch)
  ;;           (uri (string-append "https://xkbcommon.org/download/libxkbcommon-"
  ;;                               version ".tar.xz"))
  ;;           (sha256
  ;;            (base32
  ;;             "0awwz5pg9x5bj0d7dpg4a7bd4gl6k55mlpxwb12534fkrpn19p0f"))))
  (source
   (origin
     (method git-fetch)
     (uri (git-reference
           (url "https://github.com/xkbcommon/libxkbcommon.git")
           (commit (string-append "xkbcommon-" version))))
     (file-name (git-file-name name version))
     (sha256
      (base32 "13flwxxhsj8bw2isvmfxa3spma5p10kb6p9f7pc19n0my6jmjkcv"))))
  (arguments
   (substitute-keyword-arguments
       (package-arguments libxkbcommon)
     ((#:configure-flags _ #~'())
      #~(list "-Denable-x11=false" "-Denable-docs=true"))
     ))
  ;; (source (local-file "/home/declan/doc/libxkbcommon-website/download/libxkbcommon-1.7.0.tar.xz"))
  (inputs (modify-inputs (package-inputs libxkbcommon)
	    (delete "libx11")))))

(define vcs-file?
  ;; Return true if the given file is under version control.
  (or (git-predicate (current-source-directory))
      (const #t)))                                ;not in a Git checkout

(define emacs-wlc
  (package/inherit emacs-next
    (name "emacs-wlc")
    (version "31.0.50")
    (source (local-file "." "guile-checkout"
			#:recursive? #t
			#:select? vcs-file?))
    (arguments
     (substitute-keyword-arguments
	 (package-arguments emacs-next)
       ((#:configure-flags flags #~'())
	#~(cons* "--with-modules" "--with-native-compilation=aot"
		 (delete "--with-gnutls=no" #$flags)))
       ))

    (inputs (modify-inputs (package-inputs emacs-no-x)
	      (prepend
               dbus
	       wayland
	       wayland-protocols
	       mesa
	       libwebp
	       libxkbcommon-1.7)))
    (native-inputs (modify-inputs (package-native-inputs emacs-next)
		     (prepend
		      (rustup "nightly-2025-01-25")
		      clang-toolchain ;; required for bindgen from emacs-sys   Unable to generate bindings: ClangDiagnostic("../../src/config.h:3754:13: fatal error: 'stdbool.h' file not found\n")
		      `(glibc ,debug)
		      )))))

(define emacs-pgtk-wr
  (package/inherit emacs-next-pgtk
    (name "emacs-pgtk-wr")
    (version "31.0.50")
    (source (local-file "." "guile-checkout"
			#:recursive? #t
			#:select? vcs-file?))
    (arguments
     (substitute-keyword-arguments
	 (package-arguments emacs-next)
       ((#:configure-flags flags #~'())
	#~(cons* "--with-modules" "--with-native-compilation=aot"
		 "--with-pgtk" "--with-webrender"
		 (delete "--with-gnutls=no" #$flags)))
       ))

    (inputs (modify-inputs (package-inputs emacs-next-pgtk)
	      (prepend
               dbus
	       wayland
	       wayland-protocols
	       mesa
	       libwebp
	       libxkbcommon-1.7)))
    (native-inputs (modify-inputs (package-native-inputs emacs-next-pgtk)
		     (prepend
		      (rustup "nightly-2025-01-25")
		      clang-toolchain ;; required for bindgen from emacs-sys   Unable to generate bindings: ClangDiagnostic("../../src/config.h:3754:13: fatal error: 'stdbool.h' file not found\n")
		      ;; and for linker cc
		      gcc-toolchain ;; webrender-c04a6b14b677b8ea/build-script-build: error while loading shared libraries: libstdc++.so.6: cannot open shared object file: No such file or directory
		      )))))

emacs-pgtk-wr
