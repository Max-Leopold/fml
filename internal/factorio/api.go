package factorio

import (
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"sort"
	"time"

	"github.com/lithammer/fuzzysearch/fuzzy"
)

const ApiUrl = "https://mods.factorio.com/"

type Mod struct {
	Category       string    `json:"category,omitempty"`
	Changelog      string    `json:"changelog,omitempty"`
	CreatedAt      time.Time `json:"created_at,omitempty"`
	Description    string    `json:"description,omitempty"`
	DownloadsCount int       `json:"downloads_count,omitempty"`
	Faq            string    `json:"faq,omitempty"`
	GithubPath     string    `json:"github_path,omitempty"`
	Homepage       string    `json:"homepage,omitempty"`
	Images         []Images  `json:"images,omitempty"`
	License        License   `json:"license,omitempty"`
	Name           string    `json:"name,omitempty"`
	Owner          string    `json:"owner,omitempty"`
	Releases       []Release `json:"releases,omitempty"`
	LatestRelease  *Release  `json:"latest_release,omitempty"`
	Score          float64   `json:"score,omitempty"`
	SourceURL      string    `json:"source_url,omitempty"`
	Summary        string    `json:"summary,omitempty"`
	Tag            Tag       `json:"tag,omitempty"`
	Thumbnail      string    `json:"thumbnail,omitempty"`
	Title          string    `json:"title,omitempty"`
	UpdatedAt      time.Time `json:"updated_at,omitempty"`
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
type Release struct {
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

func (m *Mod) GetLatestRelease() Release {
	if m.LatestRelease == nil {
		if len(m.Releases) > 0 {
			return m.Releases[len(m.Releases)-1]
		}

		return Release{}
	}

	return *m.LatestRelease
}

type modList struct {
	Mods []Mod `json:"results"`
}

func SearchMods(query string) []Mod {
	// TODO This should be cached somehow
	req := NewModsRequest()
	req.PageSize = "max"
	mods := req.Execute()

	matchingMods := mods[:0]
	for _, mod := range mods {
		match := fuzzy.MatchNormalizedFold(query, mod.Name)
		if match {
			matchingMods = append(matchingMods, mod)
		}
	}
	sort.Slice(matchingMods, func(i, j int) bool {
		return matchingMods[i].DownloadsCount > matchingMods[j].DownloadsCount
	})

	return matchingMods
}

func GetModsFromConfig(modConfig ModConfig) []Mod {
	names := make([]string, len(modConfig.Mods))
	for i := range modConfig.Mods {
		names[i] = modConfig.Mods[i].Name
	}

	req := NewModsRequest()
	req.NameList = &names

	return req.Execute()
}

func DownloadModsFromConfig(downloadDirectory string, modConfig ModConfig, serverConfig ServerConfig) {
	client := &http.Client{}
	if _, err := os.Stat(downloadDirectory); os.IsNotExist(err) {
		err := os.Mkdir(downloadDirectory, os.ModePerm)
		if err != nil {
			log.Fatal(err)
		}
	}

	for _, mod := range GetModsFromConfig(modConfig) {
		fileName := downloadDirectory + "/" + mod.GetLatestRelease().FileName

		fmt.Println("Downloading " + mod.Name + " to " + fileName)
		if _, err := os.Stat(fileName); err == nil {
			fmt.Println(mod.Name + " already exists -  skipping")
			continue
		}

		req, err := http.NewRequest("GET", ApiUrl+"/"+mod.GetLatestRelease().DownloadURL, nil)
		if err != nil {
			log.Fatal(err)
		}

		query := req.URL.Query()
		query.Add("token", serverConfig.Token)
		query.Add("username", serverConfig.Username)

		req.URL.RawPath = query.Encode()

		res, err := client.Do(req)
		if err != nil {
			log.Fatal(err)
		}
		defer res.Body.Close()

		out, err := os.Create(fileName)
		if err != nil {
			log.Fatal(err)
		}
		defer out.Close()

		_, err = io.Copy(out, res.Body)
		if err != nil {
			log.Fatal(err)
		}
	}
}

func parseMod(modJson *[]byte) Mod {
	var mod Mod
	json.Unmarshal(*modJson, &mod)

	return mod
}

func parseModList(modListJson *[]byte) modList {
	var modList modList
	json.Unmarshal(*modListJson, &modList)

	return modList
}

func deleteMod(downloadDirectory string, modName string) {
	files, err := filepath.Glob(downloadDirectory + modName)
	if err != nil {
		log.Fatal(err)
	}

	for _, file := range files {
		if err := os.Remove(file); err != nil {
			log.Fatal(err)
		}
	}
}
