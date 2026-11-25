# Project references and maintenance

IBM i2 Analyst's Notebook documentation describes charts built from typed entities and links, attributes on chart items, confidence and direction on links, highly connected item analysis, and timeline charts. IBM identifies `.anx` as the standardized XML chart exchange and `.anb` as the native chart file. Arch Notebook now imports supported ANX chart items and exports an initial ANX representation; IBM XSD validation and application-level interoperability review remain open. The ANX schema and examples are distributed with an installed Analyst's Notebook, and GraphML does not replace ANX. These concepts inform the chart, investigation, and interoperability workflow:

- [IBM Analyst's Notebook 10.1.0 import formats](https://docs.i2group.com/anb/10.1.0/import_formats.html)
- [IBM Analyst's Notebook file types](https://docs.i2group.com/anb/10.1.2/file_types.html)
- [IBM Analyst's Notebook: listing linked entities](https://docs.i2group.com/anb/10.1.2/list_items_sort.html)
- [IBM Analyst's Notebook: finding items linked to a selection](https://docs.i2group.com/anb/10.0.2/find_linked.html)
- [IBM Analyst's Notebook Select functions](https://docs.i2group.com/anb/10.1.1/anb_select_menu.html)
- [IBM Analyst's Notebook link styles and confidence](https://docs.i2group.com/anb/10.0.1/about_links.html)
- [IBM support note: locating the ANX schema and examples](https://www.ibm.com/support/pages/node/546669)
- [IBM Analyst's Notebook User Help (9.2.4)](https://www.ibm.com/docs/en/SSJSV9_9.2.4/pdf/analysts_notebook_pdf.pdf)
- [IBM Analyst's Notebook Essentials](https://www.ibm.com/training/course/ibm-i2-analysts-notebook-essentials-9I02G)
- [SQLite read-only open flags](https://www.sqlite.org/c3ref/open.html)
- [rusqlite Connection API](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html)

Documentation and maintenance passes follow the generalized [repository sweep decision-routing policy](https://github.com/pskeffington/agent_skills/blob/main/docs/REPO_SWEEP_DECISION_ROUTING.md). Build, packaging, hardware, visual, and release claims remain separately evidenced; a passing source build does not by itself establish interactive or production acceptance.
