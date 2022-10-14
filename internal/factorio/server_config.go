package factorio

import (
	"encoding/json"
	"io/ioutil"
	"log"
)

type ServerConfig struct {
	Token    string `json:"token"`
	Username string `json:"username"`
}

func GetServerConfig(path string) ServerConfig {
	return readServerConfig(path)
}

func readServerConfig(path string) ServerConfig {
	file, err := ioutil.ReadFile(path)
	if err != nil {
		log.Fatal(err)
	}

	var config ServerConfig
	json.Unmarshal([]byte(file), &config)

	return config
}
