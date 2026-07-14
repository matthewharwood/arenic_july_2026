document.addEventListener("alpine:init", () => {
  Alpine.data("ecsDashboard", () => ({
    activeQuery: 0,
    dashboard: null,
    marks: { read: "R", write: "W", with: "+", without: "−", optional: "?" },

    init() {
      this.dashboard = JSON.parse(this.$root.dataset.dashboard);
    },

    get query() {
      return this.dashboard.queries[this.activeQuery];
    },

    get matchCount() {
      return [...this.$root.querySelectorAll("#sheet-body tr")]
        .filter((_, rowIndex) => this.matches(rowIndex)).length;
    },

    present(rowIndex, component) {
      return this.$root
        .querySelectorAll("#sheet-body tr")[rowIndex]
        .querySelector('[data-component="' + component + '"]').dataset.present === "true";
    },

    matches(rowIndex) {
      return this.query.rules.every((rule) => {
        const present = this.present(rowIndex, rule.component);
        if (rule.mode === "without") return !present;
        return rule.mode === "optional" || present;
      });
    },

    rule(component) {
      return this.query.rules.find((rule) => rule.component === component);
    },

    cellActive(rowIndex, component) {
      return this.matches(rowIndex) && Boolean(this.rule(component));
    },

    cellState(rowIndex, component) {
      return this.cellActive(rowIndex, component) ? "active" : "inactive";
    },

    cellMode(rowIndex, component) {
      return this.cellActive(rowIndex, component) ? this.rule(component).mode : "";
    },

    accessMark(component) {
      return this.marks[this.rule(component)?.mode] ?? "";
    },
  }));
});
