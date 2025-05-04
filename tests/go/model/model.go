package model

type Cab struct {
	Id       int
	Location int
	Status   string
}

type Stop struct {
	Id int
	//No 		string
	Name string
	//Type 	string
	Bearing   string
	Latitude  float64
	Longitude float64
}

type Route struct {
	Id     int
	Status string
	Legs   []Task
}

type Task struct {
	Id     int
	From   int
	To     int
	Place  int
	Status string
}

type Demand struct {
	Id     int
	From   int
	To     int
	Eta    int // set when assigned
	Shared bool
	InPool bool
	Cab    Cab
	Status string
	Wait   int // max wait for assignment
	Loss   int // [%] loss in Pool
	// LocalDateTime atTime;
	Distance int
	//Received time.Time
}
