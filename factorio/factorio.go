package factorio

const (
	ApiUrl = "https://mods.factorio.com/"
)

type FactorioRequest interface {
	Execute() *[]Mod
}

