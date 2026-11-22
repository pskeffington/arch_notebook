DROP TABLE IF EXISTS connection_observations;
CREATE TABLE connection_observations (
    snapshot_id TEXT NOT NULL,
    connection_id TEXT NOT NULL,
    source TEXT NOT NULL,
    source_type TEXT NOT NULL,
    target TEXT NOT NULL,
    target_type TEXT NOT NULL,
    relationship TEXT NOT NULL,
    event_date TEXT NOT NULL,
    confidence INTEGER NOT NULL CHECK (confidence BETWEEN 1 AND 3),
    directed INTEGER NOT NULL DEFAULT 1,
    source_reference TEXT NOT NULL,
    notes TEXT NOT NULL DEFAULT ''
);

INSERT INTO connection_observations VALUES
('baseline', 'C-101', 'Mara Venn', 'PERSON', 'North Quay Storage', 'LOCATION', 'VISITED', '2025-01-12', 3, 1, 'CASE-0042', 'Routine visit'),
('baseline', 'C-102', 'Elias Ward', 'PERSON', 'North Quay Storage', 'LOCATION', 'VISITED', '2025-01-13', 3, 1, 'CASE-0047', 'Vehicle observed'),
('baseline', 'C-103', 'Mara Venn', 'PERSON', 'Oriole Freight', 'ORGANIZATION', 'CONTACTED', '2025-01-15', 2, 1, 'CASE-0051', 'Call record'),
('baseline', 'C-104', 'Elias Ward', 'PERSON', 'Oriole Freight', 'ORGANIZATION', 'PAID', '2025-01-16', 2, 1, 'CASE-0058', 'Invoice link'),
('current', 'C-101', 'Mara Venn', 'PERSON', 'North Quay Storage', 'LOCATION', 'VISITED', '2025-01-12', 3, 1, 'CASE-0042', 'Routine visit'),
('current', 'C-102', 'Elias Ward', 'PERSON', 'North Quay Storage', 'LOCATION', 'VISITED', '2025-01-13', 1, 1, 'CASE-0047', 'Vehicle observation downgraded'),
('current', 'C-103', 'Mara Venn', 'PERSON', 'Oriole Freight', 'ORGANIZATION', 'CONTACTED', '2025-01-15', 2, 1, 'CASE-0051', 'Call record'),
('current', 'C-105', 'Mara Venn', 'PERSON', 'Eastbridge Depot', 'LOCATION', 'VISITED', '2025-01-22', 2, 1, 'CASE-0064', 'New observation');
