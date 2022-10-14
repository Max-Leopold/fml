package factorio

import (
	"encoding/json"
	"io/ioutil"
	"log"
)

type ModConfig struct {
	Mods []struct {
		Name    string `json:"name"`
		Enabled bool   `json:"enabled"`
	} `json:"mods"`
}

func GetModConfig(path string) ModConfig {
	return readModConfig(path)
}

func readModConfig(path string) ModConfig {
	file, err := ioutil.ReadFile(path)
	if err != nil {
		log.Fatal(err)
	}

	var config ModConfig
	json.Unmarshal([]byte(file), &config)

	return config
}

