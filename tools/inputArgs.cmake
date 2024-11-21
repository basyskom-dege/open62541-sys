# List of nodesets to generate
set(NODESETS "DI" "AUTOID")

# May require hardcoded paths. This is just an example
# Arguments for the DI nodeset
set(DI_NAME "di")
set(DI_FILE_CSV "../open62541/deps/ua-nodeset/DI/Opc.Ua.Di.NodeIds.csv")
set(DI_FILE_NS "../open62541/deps/ua-nodeset/DI/Opc.Ua.Di.NodeSet2.xml")
set(DI_FILE_BSD "../open62541/deps/ua-nodeset/DI/Opc.Ua.Di.Types.bsd")
set(DI_OUTPUT_DIR "../src/nodesets/di")
set(DI_INTERNAL "INTERNAL")

# Arguments for the AUTOID nodeset
set(AUTOID_NAME "autoid")
set(AUTOID_FILE_NS "../open62541/deps/ua-nodeset/AutoID/Opc.Ua.AutoID.NodeSet2.xml")
set(AUTOID_FILE_CSV "../open62541/deps/ua-nodeset/AutoID/Opc.Ua.AutoID.NodeIds.csv")
set(AUTOID_OUTPUT_DIR "../src/nodesets/autoid")
set(AUTOID_DEPENDS "di")
set(AUTOID_INTERNAL "INTERNAL")
