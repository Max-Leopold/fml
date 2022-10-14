package factorio_api

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"time"
)

const ApiUrl = "https://mods.factorio.com/api/mods"

type Mod struct {
	Category       string     `json:"category,omitempty"`
	Changelog      string     `json:"changelog,omitempty"`
	CreatedAt      time.Time  `json:"created_at,omitempty"`
	Description    string     `json:"description,omitempty"`
	DownloadsCount int        `json:"downloads_count,omitempty"`
	Faq            string     `json:"faq,omitempty"`
	GithubPath     string     `json:"github_path,omitempty"`
	Homepage       string     `json:"homepage,omitempty"`
	Images         []Images   `json:"images,omitempty"`
	License        License    `json:"license,omitempty"`
	Name           string     `json:"name,omitempty"`
	Owner          string     `json:"owner,omitempty"`
	Releases       []Releases `json:"releases,omitempty"`
	Score          float64    `json:"score,omitempty"`
	SourceURL      string     `json:"source_url,omitempty"`
	Summary        string     `json:"summary,omitempty"`
	Tag            Tag        `json:"tag,omitempty"`
	Thumbnail      string     `json:"thumbnail,omitempty"`
	Title          string     `json:"title,omitempty"`
	UpdatedAt      time.Time  `json:"updated_at,omitempty"`
}
type Images struct {
	ID        string `json:"id,omitempty"`
	Thumbnail string `json:"thumbnail,omitempty"`
	URL       string `json:"url,omitempty"`
}
type License struct {
	Description string `json:"description,omitempty"`
	ID          string `json:"id,omitempty"`
	Name        string `json:"name,omitempty"`
	Title       string `json:"title,omitempty"`
	URL         string `json:"url,omitempty"`
}
type InfoJSON struct {
	Dependencies    []string `json:"dependencies,omitempty"`
	FactorioVersion string   `json:"factorio_version,omitempty"`
}
type Releases struct {
	DownloadURL string    `json:"download_url,omitempty"`
	FileName    string    `json:"file_name,omitempty"`
	InfoJSON    InfoJSON  `json:"info_json,omitempty"`
	ReleasedAt  time.Time `json:"released_at,omitempty"`
	Sha1        string    `json:"sha1,omitempty"`
	Version     string    `json:"version,omitempty"`
}
type Tag struct {
	Name string `json:"name,omitempty"`
}

func GetMod(name string) (Mod) {
	res, err := http.Get(ApiUrl + "/" + name + "/full")
	if err != nil {
		log.Fatal(err)
	}

	defer res.Body.Close()

	body, err := ioutil.ReadAll(res.Body)
	if err != nil {
		log.Fatal(err)
	}

	return parseMod(body)
}

func parseMod(modJson []byte) (Mod) {
	var mod Mod
	json.Unmarshal(modJson, &mod)

	return mod
}

func parseMods(modArrayJson []byte) ([]Mod) {
	var mods []Mod
	json.Unmarshal(modArrayJson, &mods)

	return mods
}
