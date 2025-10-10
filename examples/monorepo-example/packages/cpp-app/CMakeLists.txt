cmake_minimum_required(VERSION 3.16)

project(cpp-app VERSION 25.10.25 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Application sources
add_executable(cpp-app
    main.cpp
)

# Version information
message(STATUS "Building cpp-app version ${PROJECT_VERSION}")

# Installation
install(TARGETS cpp-app
    RUNTIME DESTINATION bin
)