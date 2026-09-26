import json, sys
sys.path.insert(0,'.')
from mcpdrive import MCP
m = MCP("/Users/michael.pursifull/work/aae-orc/kos", "dump.stderr")
m.initialize()
r = m.request("tools/list")
for t in r["result"]["tools"]:
    if t["name"] in ("find_symbol","find_referencing_symbols","find_implementations",
                     "find_declaration","get_symbols_overview","search_for_pattern",
                     "list_memories","onboarding","activate_project","read_memory"):
        props = t["inputSchema"].get("properties",{})
        req = t["inputSchema"].get("required",[])
        print(f"### {t['name']}  required={req}")
        for k,v in props.items():
            print(f"    {k}: {v.get('type')} {str(v.get('description',''))[:110]}")
m.close()
