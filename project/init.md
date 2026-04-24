TODO 터미널 앱

- 사용하는 언어: Rust
- 데이터 저장: sqlite
    - 위치: ~/.todoj.db
    - 외부 db와 연계해서 기기간 sync 가능하게 업데이트 예정
- DB scheme:
    - id: not null
    - todo: not null
    - due_date: sqlite 에서는 yyyymmdd 형식으로 저장
    - priority: 1,2,3,4 # 기본은 3
    - up_id: 서브 프로젝트 가능, 상위 프로젝트와 연계
        - 1단계 서브만 가능. 추후 2,3 단계 확장 가능
    - done: null,1,2,3,4,5 # 5=done, 1,2,3,4는 20%,40%,60%,80%로 표시 예정
    - done_at: sqlite에서는 yyyymmdd 형식으로 저장
    - deleted_at: sqlite에서는 yyyymmddThhmmss 형식으로 저장
    - created_at: sqlite에서는 yyyymmddThhmmss 형식으로 저장
    - updated_at: sqlite에서는 yyyymmddThhmmss 형식으로 저장
- 기능
    - list
        - 리스트 순서, due_date desc, priority asc, created_at desc
        - done=5만 완료로 인정, done<4는 미완료
        - 완료된 todo는 미완료된 todo 아래에, done_at desc로
        - 완료된 todo는 안보게 할 수 있음
        - 리스트 형식은 다음처럼
          1 [ ] what to do @26-4-3(금) ^3
          2 [ ] 1> doing to do ^3 20%26-3-31(화)
          3 [x] done to o @26-4-1(수) ^1 %26-4-2(목)
        - 리스트 앞의 번호는 보이는 순서대로
        - 리스트 앞의 번호 정렬되도록. 최대가 10이면 그에 맞춰서 1,2는 앞에 공백
          추가
        - 기본적으로 상위/서브 관계 상관없이 리스트
        - order 입력하면 상위 todo에 순서를 적용하고
          서브 todo는 상위 todo 바로 밑에 순서 적용해서 리스트
    - order
        - 상위/서브 관계를 유지하면 리스트할지, 관계없이 리스트할지 스위치
    - show
        - 완료된 todo 를 리스트에서 보일지 말지 스위치
    - add
        - 다음 처럼 입력해서 처리, -d, -p, -u 없어도 되고 순서 상관없음
          add what to do -d 3/4 -p 2 -u 2
        - due_date(-d) 는 y/m/d, y-m-d, m/d, m-d, d(이번달) 로 입력 가능
        - priority(-p) 없으면 기본 3
        - (-u) 에서 입력되는 숫자는 리스트에서 보이는 숫자
        - (-u) 에서 입력되는 숫자가 서브의 서브를 만들면 자동으로 상위와 연결
    - edit
        - 입력 형식은 다음처럼 add와 동일
        - edit 1 입력하면, todo 내용 보여주고, 바로 수정 가능하도록
        - 내용 수정하고 뒤에 -d, -p, -u 넣으면 해당 항목 수정
    - remove
        - remove 1 처럼 입력
    - done
        - done 1 처럼 입력
        - done 1 2 로 뒤에 1,2,3,4,0 입력해서 완료 정도 입력/수정 가능
    - help
    - quit
        - 앱 종료
    - 모든 기능은 단축어도 가능
        - add, a
        - edit, e
        - remove, r
        - done, d
        - list, l
        - order, o
        - show, s
        - quit, q
- 실행
    - 터미널에서 todoj 실행시키면 화면 지우고 리스트가 나옴
    - 기능 입력하면 기능 처리하고, 깨끗한 화면에 리스트 나옴
    - 터미널에서 todoj -l 실행시키면 리스트만 보이고 종료
