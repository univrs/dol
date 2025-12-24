# Run biomimicry implementation  DONE
#npx claude-flow@alpha swarm "read claude-flow-biomimicry.yaml --workflow biomimicry-full"



# Run SEX system implementation
npx claude-flow@alpha swarm "read claude-flow-sex-system.yaml --workflow sex-full"



# Run specific sub-workflows
npx claude-flow@alpha swarm "read claude-flow-biomimicry.yaml --workflow mycelium-only"

npx claude-flow@alpha swarm "read claude-flow-sex-system.yaml --workflow parsing-only"
