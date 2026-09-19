export type AppPage =
  | "home"
  | "clean"
  | "insights"
  | "explore"
  | "export";

type SidebarProps = {
  activePage:
    AppPage;

  onNavigate:
    (
      page: AppPage,
    ) => void;

  hasDataset:
    boolean;
};

type NavigationItem = {
  id:
    AppPage;

  label:
    string;

  requiresDataset:
    boolean;
};

const navigationItems:
  NavigationItem[] =
[
  {
    id: "home",
    label: "Home",
    requiresDataset: false,
  },

  {
    id: "clean",
    label: "Clean",
    requiresDataset: true,
  },

  {
    id: "insights",
    label: "Insights",
    requiresDataset: true,
  },

  {
    id: "explore",
    label: "Explore",
    requiresDataset: true,
  },

  {
    id: "export",
    label: "Export",
    requiresDataset: true,
  },
];

export function Sidebar(
  {
    activePage,
    onNavigate,
    hasDataset,
  }: SidebarProps,
) {
  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <div className="sidebar-logo">
          F
        </div>

        <span>
          Flowtren
        </span>
      </div>

      <nav className="sidebar-nav">
        {navigationItems.map(
          (
            item,
          ) => {
            const disabled =
              item.requiresDataset
              && !hasDataset;

            const active =
              activePage
              === item.id;

            return (
              <button
                key={
                  item.id
                }
                type="button"
                disabled={
                  disabled
                }
                className={
                  [
                    "sidebar-item",

                    active
                      ? "sidebar-item-active"
                      : "",

                    disabled
                      ? "sidebar-item-disabled"
                      : "",
                  ]
                    .filter(
                      Boolean,
                    )
                    .join(
                      " ",
                    )
                }
                onClick={
                  () => {
                    if (
                      !disabled
                    ) {
                      onNavigate(
                        item.id,
                      );
                    }
                  }
                }
              >
                {
                  item.label
                }
              </button>
            );
          },
        )}
      </nav>

      <div className="sidebar-footer">
        <span className="beta-badge">
          Beta
        </span>

        <span>
          v0.1.0-beta.1
        </span>
      </div>
    </aside>
  );
}