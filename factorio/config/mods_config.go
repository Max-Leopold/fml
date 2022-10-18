package config

import (
	"encoding/json"
	"io/ioutil"
	"log"

	"github.com/Max-Leopold/factorio-mod-loader/factorio"
	"github.com/Max-Leopold/factorio-mod-loader/factorio/requests"
)

type ModConfig struct {
	Mods []struct {
		Name    string `json:"name"`
		Enabled bool   `json:"enabled"`
	} `json:"mods"`
}

func GetModsFromConfig(modConfig ModConfig) *[]factorio.Mod {
	names := make([]string, len(modConfig.Mods))
	for i := range modConfig.Mods {
		names[i] = modConfig.Mods[i].Name
	}

	req := requests.NewModsRequest()
	req.NameList = &names

	return req.Execute()
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
