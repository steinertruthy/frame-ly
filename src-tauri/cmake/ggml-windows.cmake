# Included right after whisper.cpp's `project(ggml)` (CMAKE_PROJECT_ggml_INCLUDE, set in
# .cargo/config.toml). Only acts when the *target* is Windows:
#
# - whisper-rs-sys decides what to link with `cfg!` on the build host, so a Windows build made on
#   macOS asks for `ggml-blas` (an Apple-only backend). An empty library satisfies that request.
# - Cross-compiling turns GGML_NATIVE off, which would leave only SSE; enable the AVX2 family
#   (FrameLy checks for AVX2 at runtime before loading a model).
if(WIN32)
  set(GGML_AVX ON CACHE BOOL "" FORCE)
  set(GGML_AVX2 ON CACHE BOOL "" FORCE)
  set(GGML_FMA ON CACHE BOOL "" FORCE)
  set(GGML_F16C ON CACHE BOOL "" FORCE)
  file(WRITE "${CMAKE_BINARY_DIR}/framely_ggml_blas_placeholder.c" "void framely_ggml_blas_placeholder(void) {}\n")
  add_library(ggml-blas STATIC "${CMAKE_BINARY_DIR}/framely_ggml_blas_placeholder.c")
  install(TARGETS ggml-blas ARCHIVE DESTINATION lib)
endif()
