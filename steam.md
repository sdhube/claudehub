graph TD
    %% הגדרת סגנונות
    classDef israel fill:#f9dcdc,stroke:#a00,stroke-width:2px;
    classDef judah fill:#dcdcf9,stroke:#00a,stroke-width:2px;
    classDef central fill:#fffacd,stroke:#daa520,stroke-width:2px;

    %% ממלכת ישראל
    subgraph Israel ["ממלכת ישראל"]
        A[אחאב]:::israel --- B[איזבל]:::israel
        A & B --> C[יורם מלך ישראל]:::israel
        A & B --> D[עתליה]:::central
    end

    %% ממלכת יהודה
    subgraph Judah ["ממלכת יהודה"]
        E[יהושפט מלך יהודה]:::judah
        E --> F[יורם מלך יהודה]:::judah
    end

    %% נישואים וצאצאים
    D -. "נישואים" .-> F
    D & F --> G[אחזיהו מלך יהודה]:::central

    %% תיאור קשרים
    style D fill:#ffcc00,stroke:#333
    style G fill:#ffcc00,stroke:#333
