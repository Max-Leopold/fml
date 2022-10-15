package factorio

import (
	"io/ioutil"
	"log"
	"net/http"
	"strconv"
)

type modsRequestSortType string

var ModsRequestSort = struct {
	NAME       modsRequestSortType
	CREATED_AT modsRequestSortType
	UPDATED_AT modsRequestSortType
}{
	NAME:       "name",
	CREATED_AT: "create_at",
	UPDATED_AT: "updated:at",
}

type modsRequestSortOrderType string

var ModsRequestSortOrder = struct {
	ASCENDING  modsRequestSortOrderType
	DESCENDING modsRequestSortOrderType
}{
	ASCENDING:  "asc",
	DESCENDING: "desc",
}

type versionType string

var Version = struct {
	V0_13 versionType
	V0_14 versionType
	V0_15 versionType
	V0_16 versionType
	V0_17 versionType
	V0_18 versionType
	V1_0  versionType
	V1_1  versionType
}{
	V0_13: "0.13",
	V0_14: "0.14",
	V0_15: "0.15",
	V0_16: "0.16",
	V0_17: "0.17",
	V0_18: "0.18",
	V1_0:  "1.0",
	V1_1:  "1.1",
}

type modsRequest struct {
	HideDeprecated bool
	Page           int
	PageSize       string
	Sort           modsRequestSortType
	NameList       *[]string
	Version        versionType
}

func NewModsRequest() modsRequest {
	return modsRequest{
		HideDeprecated: true,
		Page:           0,
		PageSize:       "20",
		Sort:           ModsRequestSort.NAME,
		NameList:       nil,
		Version:        Version.V1_1,
	}
}

func (r *modsRequest) execute() []Mod {
	client := &http.Client{}
	req, err := http.NewRequest("GET", ApiUrl+"api/mods", nil)
	if err != nil {
		log.Fatal(err)
	}

	query := req.URL.Query()
	query.Add("hide_deprecated", strconv.FormatBool(r.HideDeprecated))
	query.Add("page", strconv.Itoa(r.Page))
	query.Add("page_size", r.PageSize)
	query.Add("sort", string(r.Sort))
	query.Add("version", string(r.Version))
	if r.NameList != nil {
		for _, name := range *r.NameList {
			query.Add("namelist", name)
		}
	}
	req.URL.RawQuery = query.Encode()

	res, err := client.Do(req)
	if err != nil {
		log.Fatal(err)
	}

	defer res.Body.Close()

	body, err := ioutil.ReadAll(res.Body)
	if err != nil {
		log.Fatal(err)
	}

	return parseModList(&body).Mods
}
