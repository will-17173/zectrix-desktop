import Foundation
import EventKit

// MARK: - Output helpers

func outputJSON<T: Encodable>(_ value: T) -> Never {
    let encoder = JSONEncoder()
    encoder.dateEncodingStrategy = .iso8601
    let data = try! encoder.encode(value)
    print(String(data: data, encoding: .utf8)!)
    exit(0)
}

func outputError(_ message: String) -> Never {
    fputs("ERROR: \(message)\n", stderr)
    exit(1)
}

// MARK: - Data types

struct CalendarItem: Codable {
    var externalId: String
    var title: String
    var dueDate: String?
    var isCompleted: Bool
    var lastModified: String
}

struct CalendarInfoOutput: Codable {
    var id: String
    var title: String
    var color: String?
}

struct SuccessResponse: Codable {
    var success: Bool
}

// MARK: - EventKit helpers

let store = EKEventStore()

func requestPermission(type: EKEntityType) {
    let semaphore = DispatchSemaphore(value: 0)
    var granted = false
    if #available(macOS 14.0, *) {
        store.requestFullAccessToEvents { ok, _ in granted = ok; semaphore.signal() }
    } else {
        store.requestAccess(to: type) { ok, _ in granted = ok; semaphore.signal() }
    }
    semaphore.wait()
    if !granted { outputError("permission denied") }
}

func isoString(_ date: Date?) -> String? {
    guard let d = date else { return nil }
    return ISO8601DateFormatter().string(from: d)
}

// MARK: - Commands

func cmdRequestPermission() {
    let sem = DispatchSemaphore(value: 0)
    var calGranted = false
    var remGranted = false

    if #available(macOS 14.0, *) {
        store.requestFullAccessToEvents { ok, _ in calGranted = ok; sem.signal() }
        sem.wait()
        store.requestFullAccessToReminders { ok, _ in remGranted = ok; sem.signal() }
        sem.wait()
    } else {
        store.requestAccess(to: .event) { ok, _ in calGranted = ok; sem.signal() }
        sem.wait()
        store.requestAccess(to: .reminder) { ok, _ in remGranted = ok; sem.signal() }
        sem.wait()
    }

    outputJSON(SuccessResponse(success: calGranted || remGranted))
}

func cmdListCalendars(type: String) {
    let ekType: EKEntityType = type == "reminder" ? .reminder : .event
    requestPermission(type: ekType)
    let cals = store.calendars(for: ekType)
    let result = cals.map { cal in
        let hex = cal.cgColor.map { color -> String in
            guard let components = color.components, components.count >= 3 else { return "#888888" }
            return String(format: "#%02X%02X%02X",
                Int(components[0] * 255),
                Int(components[1] * 255),
                Int(components[2] * 255))
        }
        return CalendarInfoOutput(id: cal.calendarIdentifier, title: cal.title, color: hex)
    }
    outputJSON(result)
}

func cmdListItems(calendarId: String) {
    // Try event first, then reminder
    if let cal = store.calendar(withIdentifier: calendarId), cal.allowedEntityTypes.contains(.event) {
        requestPermission(type: .event)
        let start = Date(timeIntervalSinceNow: -60 * 60 * 24 * 365)
        let end = Date(timeIntervalSinceNow: 60 * 60 * 24 * 365)
        let pred = store.predicateForEvents(withStart: start, end: end, calendars: [cal])
        let events = store.events(matching: pred)
        let items = events.map { ev -> CalendarItem in
            CalendarItem(
                externalId: ev.eventIdentifier,
                title: ev.title ?? "",
                dueDate: isoString(ev.startDate),
                isCompleted: ev.status == .done,
                lastModified: isoString(ev.lastModifiedDate ?? ev.startDate) ?? ""
            )
        }
        outputJSON(items)
    } else {
        requestPermission(type: .reminder)
        guard let cal = store.calendar(withIdentifier: calendarId) else {
            outputError("calendar not found: \(calendarId)")
        }
        let sem = DispatchSemaphore(value: 0)
        var reminders: [EKReminder] = []
        let pred = store.predicateForReminders(in: [cal])
        store.fetchReminders(matching: pred) { result in
            reminders = result ?? []
            sem.signal()
        }
        sem.wait()
        let items = reminders.map { rem -> CalendarItem in
            let due = rem.dueDateComponents?.date
            return CalendarItem(
                externalId: rem.calendarItemIdentifier,
                title: rem.title ?? "",
                dueDate: isoString(due),
                isCompleted: rem.isCompleted,
                lastModified: isoString(rem.lastModifiedDate ?? Date()) ?? ""
            )
        }
        outputJSON(items)
    }
}

struct CreateItemInput: Decodable {
    var calendarId: String
    var title: String
    var dueDate: String?
    var isCompleted: Bool
    var targetType: String // "calendar" | "reminder"
}

func cmdCreateItem(data: String) {
    let decoder = JSONDecoder()
    guard let jsonData = data.data(using: .utf8),
          let input = try? decoder.decode(CreateItemInput.self, from: jsonData)
    else { outputError("invalid JSON input") }

    guard let cal = store.calendar(withIdentifier: input.calendarId) else {
        outputError("calendar not found: \(input.calendarId)")
    }

    var externalId: String
    if input.targetType == "calendar" {
        requestPermission(type: .event)
        let ev = EKEvent(eventStore: store)
        ev.title = input.title
        ev.calendar = cal
        if let ds = input.dueDate, let date = ISO8601DateFormatter().date(from: ds) {
            ev.startDate = date
            ev.endDate = date.addingTimeInterval(3600)
        } else {
            let now = Date()
            ev.startDate = now
            ev.endDate = now.addingTimeInterval(3600)
        }
        try! store.save(ev, span: .thisEvent)
        externalId = ev.eventIdentifier
    } else {
        requestPermission(type: .reminder)
        let rem = EKReminder(eventStore: store)
        rem.title = input.title
        rem.calendar = cal
        if let ds = input.dueDate, let date = ISO8601DateFormatter().date(from: ds) {
            let comps = Calendar.current.dateComponents([.year, .month, .day, .hour, .minute], from: date)
            rem.dueDateComponents = comps
        }
        rem.isCompleted = input.isCompleted
        try! store.save(rem, commit: true)
        externalId = rem.calendarItemIdentifier
    }
    outputJSON(["externalId": externalId])
}

struct UpdateItemInput: Decodable {
    var title: String
    var dueDate: String?
    var isCompleted: Bool
}

func cmdUpdateItem(externalId: String, data: String) {
    let decoder = JSONDecoder()
    guard let jsonData = data.data(using: .utf8),
          let input = try? decoder.decode(UpdateItemInput.self, from: jsonData)
    else { outputError("invalid JSON input") }

    if let ev = store.calendarItem(withIdentifier: externalId) as? EKEvent {
        requestPermission(type: .event)
        ev.title = input.title
        if let ds = input.dueDate, let date = ISO8601DateFormatter().date(from: ds) {
            ev.startDate = date
            ev.endDate = date.addingTimeInterval(3600)
        }
        try! store.save(ev, span: .thisEvent)
    } else if let rem = store.calendarItem(withIdentifier: externalId) as? EKReminder {
        requestPermission(type: .reminder)
        rem.title = input.title
        rem.isCompleted = input.isCompleted
        if let ds = input.dueDate, let date = ISO8601DateFormatter().date(from: ds) {
            rem.dueDateComponents = Calendar.current.dateComponents([.year, .month, .day, .hour, .minute], from: date)
        }
        try! store.save(rem, commit: true)
    } else {
        outputError("item not found: \(externalId)")
    }
    outputJSON(SuccessResponse(success: true))
}

func cmdDeleteItem(externalId: String) {
    if let ev = store.calendarItem(withIdentifier: externalId) as? EKEvent {
        requestPermission(type: .event)
        try! store.remove(ev, span: .thisEvent)
    } else if let rem = store.calendarItem(withIdentifier: externalId) as? EKReminder {
        requestPermission(type: .reminder)
        try! store.remove(rem, commit: true)
    } else {
        outputError("item not found: \(externalId)")
    }
    outputJSON(SuccessResponse(success: true))
}

// MARK: - CLI dispatch

let args = CommandLine.arguments
guard args.count >= 2 else {
    fputs("Usage: calendar-bridge <command> [options]\n", stderr)
    exit(1)
}

switch args[1] {
case "request-permission":
    cmdRequestPermission()
case "list-calendars":
    let typeArg = args.first(where: { $0.hasPrefix("--type=") })?.dropFirst(7) ?? "reminder"
    cmdListCalendars(type: String(typeArg))
case "list-items":
    guard let idArg = args.first(where: { $0.hasPrefix("--calendar-id=") }) else {
        outputError("missing --calendar-id")
    }
    cmdListItems(calendarId: String(idArg.dropFirst("--calendar-id=".count)))
case "create-item":
    guard let dataArg = args.first(where: { $0.hasPrefix("--data=") }) else {
        outputError("missing --data")
    }
    cmdCreateItem(data: String(dataArg.dropFirst("--data=".count)))
case "update-item":
    guard let idArg = args.first(where: { $0.hasPrefix("--external-id=") }),
          let dataArg = args.first(where: { $0.hasPrefix("--data=") })
    else { outputError("missing --external-id or --data") }
    cmdUpdateItem(
        externalId: String(idArg.dropFirst("--external-id=".count)),
        data: String(dataArg.dropFirst("--data=".count))
    )
case "delete-item":
    guard let idArg = args.first(where: { $0.hasPrefix("--external-id=") }) else {
        outputError("missing --external-id")
    }
    cmdDeleteItem(externalId: String(idArg.dropFirst("--external-id=".count)))
default:
    outputError("unknown command: \(args[1])")
}
