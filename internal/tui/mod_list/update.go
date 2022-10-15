package modList

import tea "github.com/charmbracelet/bubbletea"

func (b bubbleMod) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {

		case "ctrl+c", "q":
			return b, tea.Quit
	}
	case tea.WindowSizeMsg:
		h, v := docStyle.GetFrameSize()
		b.list.SetSize(msg.Width-h, msg.Height-v)
	}

	var cmd tea.Cmd
	b.list, cmd = b.list.Update(msg)
	return b, cmd
}
