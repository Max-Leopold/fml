package factorio

import (
	"io/ioutil"
	"log"
	"net/http"
)

type modRequest struct {
	Name string
	Full bool
}

func NewModRequest() modRequest {
	return modRequest{
		Name: "",
		Full: true,
	}
}

func (r *modRequest) Execute() Mod {
	url := ApiUrl + "api/mods/" + r.Name
	if r.Full {
		url += "/full"
	}
	res, err := http.Get(url)
	if err != nil {
		log.Fatal(err)
	}

	defer res.Body.Close()

	body, err := ioutil.ReadAll(res.Body)
	if err != nil {
		log.Fatal(err)
	}

	return parseMod(&body)
}
