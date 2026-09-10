package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"os"

	"github.com/grafana/grafana-foundation-sdk/go/common"
	"github.com/grafana/grafana-foundation-sdk/go/dashboard"
	"github.com/grafana/grafana-foundation-sdk/go/prometheus"
	"github.com/grafana/grafana-foundation-sdk/go/stat"
	"github.com/grafana/grafana-foundation-sdk/go/statushistory"
	"github.com/grafana/grafana-foundation-sdk/go/timeseries"
)

func main() {
	output := flag.String("output", "firebird-dashboard.json", "Output file path for dashboard JSON")
	flag.Parse()

	db, err := buildFirebirdDashboard()
	if err != nil {
		log.Fatalf("Failed to build dashboard: %v", err)
	}

	jsonData, err := json.MarshalIndent(db, "", "  ")
	if err != nil {
		log.Fatalf("Failed to marshal dashboard: %v", err)
	}

	jsonData, err = fixupDashboard(jsonData)
	if err != nil {
		log.Fatalf("Failed to fixup dashboard: %v", err)
	}

	if err := os.WriteFile(*output, jsonData, 0644); err != nil {
		log.Fatalf("Failed to write dashboard file: %v", err)
	}

	fmt.Printf("Dashboard generated successfully: %s\n", *output)
}

func fixupDashboard(jsonData []byte) ([]byte, error) {
	var dashboardMap map[string]any
	if err := json.Unmarshal(jsonData, &dashboardMap); err != nil {
		return nil, err
	}

	if links, ok := dashboardMap["links"].([]any); ok && len(links) > 0 {
		if link, ok := links[0].(map[string]any); ok {
			delete(link, "placement")
		}
	}

	return json.MarshalIndent(dashboardMap, "", "  ")
}

func buildFirebirdDashboard() (dashboard.Dashboard, error) {
	ds := common.DataSourceRef{
		Type: ptrStr("prometheus"),
		Uid:  ptrStr("${DS_PROMETHEUS}"),
	}

	// Add datasource variable for selecting prometheus/victoria metrics
	dsVarBuilder := dashboard.NewDatasourceVariableBuilder("DS_PROMETHEUS").
		Type("prometheus").
		Label("Datasource")
	builder := dashboard.NewDashboardBuilder("Firebird Exporter").
		Description("Monitoring dashboard for Firebird database metrics").
		Tags([]string{"firebird", "database", "monitoring"}).
		Timezone("browser").
		Refresh("30s").
		Uid("klever-firebird").
		Link(dashboard.NewDashboardLinkBuilder("firebird exporter").
			Url("https://github.com/hacker/firebird_exporter/").
			Type(dashboard.DashboardLinkTypeLink).
			Icon("external link").
			TargetBlank(true)).
		WithVariable(dsVarBuilder)

	// Add panels
	builder = builder.WithPanel(stateTimelinePanelBuilder("Database Status", "firebird_up", ds, 0, 0, 21, 3))
	builder = builder.WithPanel(statPanelBuilder("Read-Only Mode", "firebird_database_read_only", ds, 21, 0, 3, 3))
	builder = builder.WithPanel(timeseriesPanelBuilder("Attachments", "firebird_attachments", "{{state}}", "none", ds, 0, 3))
	builder = builder.WithPanel(timeseriesPanelBuilder("Transactions", "firebird_transactions", "{{state}}", "none", ds, 12, 3))
	builder = builder.WithPanel(timeseriesPanelBuilder("Statements", "firebird_statements", "{{state}}", "none", ds, 0, 11))
	builder = builder.WithPanel(ioPanelBuilder("I/O Operations", ds, 12, 11))
	builder = builder.WithPanel(memoryPanelBuilder("Memory Usage", ds, 0, 19))
	builder = builder.WithPanel(timeseriesPanelBuilder("Transaction Throughput", "rate(firebird_database_next_transaction[5m])", "delta", "si: xact/s", ds, 0, 27))
	builder = builder.WithPanel(timeseriesPanelBuilder("Transaction ID Window", "firebird_database_next_transaction - firebird_database_oldest_active", "transactions", "none", ds, 12, 27))

	return builder.Build()
}

func stateTimelinePanelBuilder(title, expr string, ds common.DataSourceRef, x, y, w, h uint32) *statushistory.PanelBuilder {
	thresholds := dashboard.NewThresholdsConfigBuilder().
		Mode(dashboard.ThresholdsModeAbsolute).
		Steps([]dashboard.Threshold{
			{Value: ptrFloat64(0), Color: "red"},
			{Value: ptrFloat64(1), Color: "green"},
		})

	return statushistory.NewPanelBuilder().
		Title(title).
		Datasource(ds).
		GridPos(dashboard.GridPos{H: h, W: w, X: x, Y: y}).
		Unit("bool_yes_no").
		ShowValue(common.VisibilityModeNever).
		Tooltip(common.NewVizTooltipOptionsBuilder().
			Mode(common.TooltipDisplayModeSingle)).
		PerPage(0).
		AxisPlacement(common.AxisPlacementHidden).
		Thresholds(thresholds).
		Legend(common.NewVizLegendOptionsBuilder().
			ShowLegend(false)).
		WithTarget(prometheus.NewDataqueryBuilder().
			Expr(expr).
			LegendFormat("up").
			Interval("10m"))
}

func statPanelBuilder(title, expr string, ds common.DataSourceRef, x, y, w, h uint32) *stat.PanelBuilder {
	return stat.NewPanelBuilder().
		Title(title).
		Datasource(ds).
		GridPos(dashboard.GridPos{H: h, W: w, X: x, Y: y}).
		Unit("bool_on_off").
		WithTarget(prometheus.NewDataqueryBuilder().
			Expr(expr))
}

func timeseriesPanelBuilder(title, expr, legendFmt, unit string, ds common.DataSourceRef, x, y uint32) *timeseries.PanelBuilder {
	return timeseries.NewPanelBuilder().
		Title(title).
		Datasource(ds).
		GridPos(dashboard.GridPos{H: 8, W: 12, X: x, Y: y}).
		Unit(unit).
		Legend(common.NewVizLegendOptionsBuilder().
			ShowLegend(true)).
		WithTarget(prometheus.NewDataqueryBuilder().
			Expr(expr).
			LegendFormat(legendFmt))
}

func ioPanelBuilder(title string, ds common.DataSourceRef, x, y uint32) *timeseries.PanelBuilder {
	return timeseries.NewPanelBuilder().
		Title(title).
		Datasource(ds).
		GridPos(dashboard.GridPos{H: 8, W: 12, X: x, Y: y}).
		Unit("ops").
		Legend(common.NewVizLegendOptionsBuilder().
			ShowLegend(true)).
		WithTarget(prometheus.NewDataqueryBuilder().
			Expr("rate(firebird_io_page_reads_total[5m])").
			LegendFormat("read page")).
		WithTarget(prometheus.NewDataqueryBuilder().
			Expr("rate(firebird_io_page_writes_total[5m])").
			LegendFormat("write page")).
		WithTarget(prometheus.NewDataqueryBuilder().
			Expr("rate(firebird_io_page_fetches_total[5m])").
			LegendFormat("fetch page")).
		WithTarget(prometheus.NewDataqueryBuilder().
			Expr("rate(firebird_io_page_marks_total[5m])").
			LegendFormat("mark page"))
}

func memoryPanelBuilder(title string, ds common.DataSourceRef, x, y uint32) *timeseries.PanelBuilder {
	return timeseries.NewPanelBuilder().
		Title(title).
		Datasource(ds).
		GridPos(dashboard.GridPos{H: 8, W: 12, X: x, Y: y}).
		Unit("bytes").
		Legend(common.NewVizLegendOptionsBuilder().
			ShowLegend(true)).
		WithTarget(prometheus.NewDataqueryBuilder().
			Expr("firebird_memory_used_bytes").
			LegendFormat("used")).
		WithTarget(prometheus.NewDataqueryBuilder().
			Expr("firebird_memory_allocated_bytes").
			LegendFormat("allocated"))
}

func ptrStr(s string) *string {
	return &s
}

func ptrFloat64(f float64) *float64 {
	return &f
}
