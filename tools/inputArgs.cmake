# List of nodesets to generate
set(NODESETS "DI" "AUTOID")

# Arguments for the DI nodeset
set(DI_NAME "di")
set(DI_FILE_CSV "/home/mdeg/Documents/GitHub/Rust/open62541-sys/open62541/deps/ua-nodeset/DI/Opc.Ua.Di.NodeIds.csv")
set(DI_FILE_NS "/home/mdeg/Documents/GitHub/Rust/open62541-sys/open62541/deps/ua-nodeset/DI/Opc.Ua.Di.NodeSet2.xml")
set(DI_FILE_BSD "/home/mdeg/Documents/GitHub/Rust/open62541-sys/open62541/deps/ua-nodeset/DI/Opc.Ua.Di.Types.bsd")
set(DI_OUTPUT_DIR "/home/mdeg/Documents/GitHub/Rust/open62541-sys/src/nodesets/di")
set(DI_INTERNAL "INTERNAL")

# Arguments for the AUTOID nodeset
set(AUTOID_NAME "autoid")
set(AUTOID_FILE_NS "/home/mdeg/Documents/GitHub/Rust/open62541-sys/open62541/deps/ua-nodeset/AutoID/Opc.Ua.AutoID.NodeSet2.xml")
set(AUTOID_FILE_CSV "/home/mdeg/Documents/GitHub/Rust/open62541-sys/open62541/deps/ua-nodeset/AutoID/Opc.Ua.AutoID.NodeIds.csv")
set(AUTOID_OUTPUT_DIR "/home/mdeg/Documents/GitHub/Rust/open62541-sys/src/nodesets/autoid")
set(AUTOID_DEPENDS "di")
set(AUTOID_INTERNAL "INTERNAL")
