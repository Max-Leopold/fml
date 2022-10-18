package requests

import (
	"encoding/json"
	"io"
	"log"
	"net/http"

	"github.com/Max-Leopold/factorio-mod-loader/factorio"
)

type ModRequest struct {
	Name string
	Full bool
}

func NewModRequest() ModRequest {
	return ModRequest{
		Name: "",
		Full: true,
	}
}

func (r *ModRequest) Execute() *[]factorio.Mod {
	url := factorio.ApiUrl + "api/mods/" + r.Name
	if r.Full {
		url += "/full"
	}
	res, err := http.Get(url)
	if err != nil {
		log.Fatal(err)
	}

	defer res.Body.Close()

	body, err := io.ReadAll(res.Body)
	if err != nil {
		log.Fatal(err)
	}

	return &[]factorio.Mod{parseMod(&body)}
}

func parseMod(body *[]byte) factorio.Mod {
	var mod factorio.Mod
	err := json.Unmarshal(*body, &mod)
	if err != nil {
		log.Fatal(err)
	}

	return mod
}
