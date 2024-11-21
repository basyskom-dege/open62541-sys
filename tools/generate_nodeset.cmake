if(NOT DEFINED open62541_TOOLS_DIR)
    set(open62541_TOOLS_DIR "../../open62541/tools") 
endif()

if(NOT DEFINED UA_SCHEMA_DIR)
    set(UA_SCHEMA_DIR "../../open62541/deps/ua-nodeset/Schema") 
endif()


include("${CMAKE_SOURCE_DIR}/inputArgs.cmake")

# Include the script containing the ua_generate_nodeset_and_datatypes function
include("../open62541/tools/cmake/open62541Macros.cmake") 

# Initialize a list to collect targets
set(NODESET_TARGETS)

foreach(NODESET ${NODESETS})
    string(TOUPPER ${NODESET} NODESET_UPPER)
    string(TOLOWER ${NODESET} NODESET_LOWER)

    foreach(VAR NAME FILE_CSV FILE_NS FILE_BSD OUTPUT_DIR DEPENDS INTERNAL)
        set(VAR_NAME "${NODESET_UPPER}_${VAR}")
        if(DEFINED ${VAR_NAME})
            set(${VAR} "${${VAR_NAME}}")
        else()
            unset(${VAR})
        endif()
    endforeach()

    message(STATUS "Processing nodeset ${NODESET}")
    message(STATUS "  NAME: ${NAME}")
    message(STATUS "  FILE_CSV: ${FILE_CSV}")
    message(STATUS "  FILE_NS: ${FILE_NS}")
    message(STATUS "  FILE_BSD: ${FILE_BSD}")
    message(STATUS "  OUTPUT_DIR: ${OUTPUT_DIR}")
    message(STATUS "  DEPENDS: ${DEPENDS}")
    message(STATUS "  INTERNAL: ${INTERNAL}")

    ua_generate_nodeset_and_datatypes(
        NAME ${NAME}
        FILE_NS ${FILE_NS}
        FILE_CSV ${FILE_CSV}
        FILE_BSD ${FILE_BSD}
        OUTPUT_DIR ${OUTPUT_DIR}
        DEPENDS ${DEPENDS}
        ${INTERNAL}
    )


    set(NODESET_TARGETS ${NODESET_TARGETS} "open62541-generator-ns-${NAME}")
endforeach()

# Add a custom target to build all nodesets
add_custom_target(GenerateAllNodesets ALL DEPENDS ${NODESET_TARGETS})
