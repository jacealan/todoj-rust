# todoj 기획/설계 문서 (v2)

## Overview

- **프로젝트명**: todoj - Terminal TODO Manager
- **언어**: Rust
- **데이터 저장**: SQLite (_LOCAL-first_)
- **동기화**: 추후付费 기능 (PostgreSQL)

---

## Database

### Schema

```sql
CREATE TABLE todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    todo TEXT NOT NULL,
    due_date TEXT,              -- YYYYMMDD
    priority INTEGER DEFAULT 3,  -- 1,2,3,4 (기본 3)
    up_id INTEGER,              -- 상위 todo ID (서브 todo)
    done INTEGER DEFAULT 0,    -- 0,1,2,3,4,5 (5=완료)
    done_at TEXT,              -- YYYYMMDD (완료 시)
    deleted_at TEXT,           -- YYYYMMDDTHHMMSS (soft delete)
    created_at TEXT NOT NULL,  -- YYYYMMDDTHHMMSS
    updated_at TEXT NOT NULL  -- YYYYMMDDTHHMMSS
);
```

### 여러 DB 지원

```bash
todoj                         # ~/.todoj.db (기본)
todoj ~/icloud/todo.db           # Cloud backup
todoj ./work.db               # 업무용
```

---

## CLI 명령어

| 명령어 | 단축키 | 설명 |
|--------|--------|------|
| add | a,ㅁ | 새 todo 추가 |
| edit | e,ㄷ | todo 수정 |
| remove | r,ㄱ | todo 삭제 (soft delete) |
| done | d,ㅇ | 완료 상태 변경 |
| list | l,ㅣ | todo 리스트 |
| search | s,ㄴ | 키워드 검색 |
| more | m,ㅡ | todo 복제 |
| calendar | c,ㅊ | 캘린더 보기 |
| order | o,ㅐ | 상위/서브 정렬 모드 |
| past | p | 완료된 todo 보기 |
| help | h,ㅗ | 도움말 |
| quit | q,ㅂ | 종료 |

### add - 새 todo 추가

```bash
add Buy milk
add Finish report -d 3/25 -p 1           # flag 형식
add Task @today ^2                       # inline 형식
add Sub-task -u 3                        # 서브 todo
add Task @today -p 2                    # mixed
```

**옵션:**
- `-d DATE`: due date (@today, @tom, @mon, 3/15, 26/3/15 등)
- `-p PRIORITY`: priority 1-4 (기본 3)
- `-u N`: 상위 todo 번호 (creates 서브)

**inline 형식 (내용 끝):**
- `@DATE` - due date
- `^N` - priority

### edit - todo 수정

```bash
edit 1,2 -d 3/30 -p 2                  # batch edit with flags
edit 1,2 @3/30 ^2                     # batch edit with inline
edit 1 -u 3                          # change parent
edit 1 -u 0                          # 상위로 변환 (remove parent)
edit 1                                # interactive edit
```

### more - todo 복제

```bash
more 1                                  # 1번 복제
more 1,3,5                             # 여러개 복제
more 1-5                                # range 복제
more 1 -d 3/15 -p 2                     # date/priority override
more 1 @3/15 ^2                         # inline override
more 1 -u 3                             # parent 변경
more 1 -u 0                             # 상위로 변환
```

**복제 시:**
- content, priority, up_id: 동일
- due_date, done, done_at: null

### done - 완료 상태

```bash
done 1              # toggle (0 ↔ 5)
done 1 5            # complete
done 1 3            # 60% done
done 1,2,3 5       # multiple
```

---

## Date 입력

### 형식

| 입력 | 출력 | 설명 |
|-------|------|------|
| 15 | 가장 가까운 미래 15일 | day only |
| 3/15 | 해당 월/일 | this year |
| 26/3/15 | year/month/day | full |
| @today | 오늘 | keyword |
| @tom | 내일 | keyword |
| @mon | 다음 월요일 | weekday keyword |
| @오늘, @내일, @월 | Korean | keyword |

### day only 동작

- `@15` 입력 시 today=4/26 → 가장 가까운 미래 15일 (5/1)
- 1~31 사이 날짜 중 가까운 미래 날짜 자동 선택

### 오류 처리

- `@0` → "유효한 날짜가 아닙니다. 1~31 사이 날짜"
- `@33` → "33일은 없습니다. 1~31 사이 날짜"
- `@13/50` → 월/일 형식 확인 메시지

---

## List 표시

### 일반 모드

```
1 [ ] Parent task     @26-03-20 ^3
2 [ ] 1> Sub-task   @26-03-21 ^3
3 [x] Completed    %26-03-19 ^2
```

**형식:**
- `N` - 번호 (표시 순서)
- `[ ]` / `[x]` - 미완료/완료
- `N>` - parent reference (회색)
- `@YY-MM-DD` - due date
- `%XX` - 진행률 (20%, 40%, 60%, 80%)
- `%YY-MM-DD` - 완료일
- `^N` - priority

### order 모드 (o)

```
1 [ ] Parent task     ^3
   [ ] Sub-task 1   ^3
   [ ] Sub-task 2   ^3
2 [ ] Another       ^2
```

**order 모드 특징:**
- 상위 todo 먼저 표시
- 서브 todo는 들여쓰기 (2 spaces)
- parent가 completed면 child 위에 표시
- parent가 deleted면 `x>`로 표시, 하단에 standalone으로

### deleted parent 처리

- parent가 deleted: `x>`로 표시 (gray)
- `-u 0`으로 parent 제거 가능

---

## Color

### due date (@)

| 색상 | 의미 |
|------|------|
| 빨강 | today 또는 overdue |
| 노랑 | 7일 이내 |
| 초록 | 미래 |

### priority (^)

| 색상 | priority |
|------|----------|
| 주황 | 1 (highest) |
| 파랑 | 2 |
| 초록 | 3 (default) |
| 회색 | 4 (lowest) |

---

## Keyboard Shortcuts

| 키보드 | 명령어 | 설명 |
|--------|--------|------|
| ㅁ | add | 새 todo |
| ㄷ | edit | 수정 |
| ㄱ | remove | 삭제 |
| ㅇ | done | 완료 |
| ㅣ | list | 리스트 |
| ㅊ | calendar | 캘린더 |
| ㅐ | order | 정렬 모드 |
| ㄴ | search | 검색 |
| p | past | 완료 보기 |
| ㅗ | help | 도움말 |
| ㅂ | quit | 종료 |
| m,ㅡ | more | 복제 |

---

## 기능 구현 상태

| 기능 | 상태 |
|------|------|
| add | ✅ |
| edit (batch + interactive) | ✅ |
| remove (soft delete) | ✅ |
| done (0-5 level) | ✅ |
| list (정렬, pagination) | ✅ |
| search (pagination) | ✅ |
| more (복제) | ✅ |
| calendar (single month + 4 weeks) | ✅ |
| order (상위/서브 정렬) | ✅ |
| order (deleted parent 처리) | ✅ |
| past (완료 보기) | ✅ |
| help | ✅ |
| -u N (parent) | ✅ |
| -u 0 (remove parent) | ✅ |
| @date inline | ✅ |
| ^priority inline | ✅ |
| Korean shortcuts | ✅ |
| 여러 DB 파일 | ✅ |
| Date validation | ✅ |
| day only future date | ✅ |

---

## 추후 개발 사항

- [ ] Web interface
- [ ] PostgreSQL server sync (付费)
- [ ] 태그/카테고리
- [ ] 반복 todo
- [ ] 다중 계정

---

## 변경 이력 (v1 → v2)

1. **more 명령어 추가**: 기존 todo 복제
2. **search 명령어 추가**: 키워드 검색
3. **order 모드 개선**: deleted parent 처리 (`x>`)
4. **-u 0**: 서브 → 상위 변환
5. **day only**: 가장 가까운 미래 날짜로 변경
6. **Date validation**: day/month 각각 validation
7. **여러 DB**: positional argument로 DB 경로 지정
8. **-inline @/^**: add/edit/more에서 모두 지원
9. **Korean shortcuts**: 전체 지원
10. **recursive parent**: 서브의 서브 지원 (order 모드)